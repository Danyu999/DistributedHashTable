mod server_functions;

use mylib::common::properties::{Properties, get_properties};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{confirm_distributed_barrier_server, handle_client_checks};
use server_functions::{accept_client};
use std::sync::{Arc, Mutex};
use mylib::common::threadpool::ThreadPool;
use std::thread;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use mylib::common::metrics::{Metrics, gather_metrics};
use mylib::common::logger::Log;

fn main() {
    let properties: Properties = get_properties();
    let num_buckets : usize = ((properties.key_range[1] - properties.key_range[0]) * 2) as usize;
    let mut hashtable = Arc::new(Hashtable::new(num_buckets));
    let _pool = ThreadPool::new(properties.dht_num_threads);

    // Get the logger file for the client
    let logger : Arc<Mutex<Log>> = Arc::new(Mutex::new(Log::server_new()));

    // Does the distributed barrier, ensuring all servers are up and ready before continuing
    // Note: Decided thread pool was overkill for distributed barrier, so it just spawns a few threads
    if !confirm_distributed_barrier_server(&properties.server_port, &properties.node_ips) {
        panic!("Distributed barrier for server failed!");
    }

    let server_client_check_port_copy = properties.server_client_check_port.clone();

    //spawn a thread to handle new clients that want to confirm that the server is up
    thread::spawn(move || { handle_client_checks(&server_client_check_port_copy) });

    //spawn a thread to gather metrics throughout the lifetime of the server
    let metrics_list: Vec<Metrics> = Vec::new();
    let threads_metrics: Arc<Metrics> = Arc::new(Metrics::new());
    let threads_metrics_clone = threads_metrics.clone();
    let stop_server = Arc::new(AtomicBool::new(false));
    let stop_server_clone = stop_server.clone();
    let stop_server_followup = Arc::new(AtomicBool::new(false));
    let stop_server_followup_clone = stop_server_followup.clone();

    thread::spawn(move || { gather_metrics(metrics_list, threads_metrics, stop_server_clone, stop_server_followup_clone) });

    // Set handler for ctrlc interrupt that will print the metrics before exiting
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        stop_server.store(true, Relaxed);
    }).expect("Error setting Ctrl-C handler");

    //let func = |stream| {return handle_client(stream, hashtable, &properties)}; //TODO: refactor to make the closure method work
    println!("Starting accept_client");
    thread::spawn(move || {accept_client(&properties.server_port, &mut hashtable, &_pool, threads_metrics_clone, logger); });

    // Spin and check the stop_server value
    loop {
        if stop_server_followup.load(Relaxed) {
            println!("Server shutting down...");
            return;
        }
    }
}