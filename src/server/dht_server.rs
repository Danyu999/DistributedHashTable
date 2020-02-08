mod server_functions;

use mylib::common::properties::{Properties, get_properties};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::confirm_distributed_barrier;
use server_functions::{accept_client};
use std::sync::{Arc, Mutex};
use mylib::common::threadpool::ThreadPool;

fn main() {
    let properties: Properties = get_properties();

    // Does the distributed barrier, ensuring all servers are up and ready before continuing
    // Note: Decided thread pool was overkill for distributed barrier, so it just spawns a few threads
    if !confirm_distributed_barrier(&properties.server_port, &properties.node_ips, true) {
        panic!("Distributed barrier for server failed!");
    }

    let mut hashtable = Arc::new(Mutex::new(Hashtable::new(properties.dht_num_buckets)));
    let pool= ThreadPool::new(properties.dht_num_threads);
    //let func = |stream| {return handle_client(stream, hashtable, &properties)}; //TODO: refactor to make the closure method work
    accept_client(&mut hashtable, &pool);
}