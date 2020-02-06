mod server_functions;

use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use mylib::common::properties::{Properties, getProperties};
use mylib::common::hashtable::Hashtable;
use server_functions::{accept_client, handle_client};

fn main() {
    let properties: Properties = getProperties();
    //TODO: create a thread pool
    let mut hashtable = Hashtable::new();
    //let func = |stream| {return handle_client(stream, hashtable, &properties)};
    accept_client(&properties, &mut hashtable);
}