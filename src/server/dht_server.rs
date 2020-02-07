mod server_functions;

use mylib::common::properties::{Properties, get_properties};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::confirm_distributed_barrier;
use server_functions::{accept_client};

fn main() {
    let properties: Properties = get_properties();
    //TODO: create a thread pool to be used for confirm_distributed_barrier and accept_client
    if !confirm_distributed_barrier(&properties.server_port, &properties.node_ips, true) {
        panic!("Distributed barrier for server failed!");
    }

    let mut hashtable: Hashtable<String> = Hashtable::new();
    //let func = |stream| {return handle_client(stream, hashtable, &properties)}; //TODO: refactor to make the closure method work
    accept_client(&properties.server_port, &properties.node_ips, &mut hashtable);
}