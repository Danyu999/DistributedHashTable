mod server_functions;

use mylib::common::properties::{Properties, get_properties};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::confirm_distributed_barrier;
use server_functions::{accept_client};
use std::sync::Arc;

fn main() {
    let properties: Properties = get_properties();
    if !confirm_distributed_barrier(&properties.server_port, &properties.node_ips, true) {
        panic!("Distributed barrier failed!");
    }

    //TODO: create a thread pool
    let mut hashtable = Hashtable::new();
    //let func = |stream| {return handle_client(stream, hashtable, &properties)};
    accept_client(&properties.server_port, &properties.node_ips, &mut hashtable);
}