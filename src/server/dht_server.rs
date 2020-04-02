mod server_functions;

use mylib::common::properties::{Properties, get_properties};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{confirm_distributed_barrier_server, handle_client_checks};
use server_functions::{accept_client};
use std::sync::Arc;
use mylib::common::threadpool::ThreadPool;
use std::thread;

fn main() {
    let properties: Properties = get_properties();
    let mut hashtable = Arc::new(Hashtable::new(properties.dht_num_buckets));
    let pool = ThreadPool::new(properties.dht_num_threads);

    // Does the distributed barrier, ensuring all servers are up and ready before continuing
    // Note: Decided thread pool was overkill for distributed barrier, so it just spawns a few threads
    if !confirm_distributed_barrier_server(&properties.server_port, &properties.node_ips) {
        panic!("Distributed barrier for server failed!");
    }

    let server_client_check_port_copy = properties.server_client_check_port.clone();

    //spawn a thread to handle new clients that want to confirm that the server is up
    thread::spawn(move || { handle_client_checks(&server_client_check_port_copy) });

    //let func = |stream| {return handle_client(stream, hashtable, &properties)}; //TODO: refactor to make the closure method work
    accept_client(&properties.server_port, &mut hashtable, &pool);
}