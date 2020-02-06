mod server_functions;

use mylib::common::properties::{Properties, get_properties};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::confirm_distributed_barrier;
use server_functions::{accept_client};

fn main() {
    let properties: Properties = get_properties();
    //TODO: implement distributed barrier
    if !confirm_distributed_barrier(&properties.server_port, &properties.node_ips, true) {
        return;
    }

    //TODO: create a thread pool
    let mut hashtable = Hashtable::new();
    //let func = |stream| {return handle_client(stream, hashtable, &properties)};
    accept_client(&properties.server_port, &properties.node_ips, &mut hashtable);
}