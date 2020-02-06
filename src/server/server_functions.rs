use std::net::{TcpStream, Shutdown, TcpListener, SocketAddr};
use mylib::common::hashtable::Hashtable;
use std::io::{Write, Read};
use mylib::common::properties::Properties;
use mylib::common::net::MessageType;
use std::thread;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

pub fn handle_client(mut stream: TcpStream, properties: &Properties, hashtable: &mut Hashtable, msg: MessageType<T>) {
    println!("Properties num nodes: {}", properties.NUM_NODES);
    //TODO: Need to do something with the hashtable based on the msg

}

pub fn accept_client(properties: &Properties, hashtable: &mut Hashtable) {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        match readMessageFromStream(&stream) {
            Ok(mut msg) => {
                handle_client(stream.0, &properties, hashtable, msg);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    // close the socket server
    drop(listener);
}