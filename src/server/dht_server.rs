mod server_functions;

use mylib::common::properties::{Properties, get_properties};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::confirm_distributed_barrier;
use server_functions::{accept_client};
use std::sync::{Arc, Mutex};

fn main() {
    let properties: Properties = get_properties();
    //TODO: create a thread pool to be used for confirm_distributed_barrier and accept_client
    if !confirm_distributed_barrier(&properties.server_port, &properties.node_ips, true) {
        panic!("Distributed barrier for server failed!");
    }

    let mut hashtable = Arc::new(Mutex::new(Hashtable::new()));
    //let func = |stream| {return handle_client(stream, hashtable, &properties)}; //TODO: refactor to make the closure method work
    accept_client(&mut hashtable);
}