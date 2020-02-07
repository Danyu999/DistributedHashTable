use std::net::{TcpStream, TcpListener, Ipv4Addr};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream};

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

pub fn handle_client(stream: TcpStream, server_port: &u64, node_ips: &Vec<Ipv4Addr>, hashtable: &mut Hashtable<String>, msg: DHTMessage) {
    println!("Server port: {}", &server_port);
    println!("First node ip: {}", &node_ips[0]);
    //TODO: Need to do something with the hashtable based on the msg
    let request = DHTMessage::Response(true);
    serde_json::to_writer(&stream, &request).unwrap();
}

pub fn accept_client(server_port: &u64, node_ips: &Vec<Ipv4Addr>, hashtable: &mut Hashtable<String>) {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        println!("Connection accepted! Handling...");
        match stream {
            Ok(stream) => {
                match read_request_message_from_stream(&stream) {
                    Ok(msg) => { handle_client(stream, &server_port, &node_ips, hashtable, msg); println!("Done handling, listening again...") }
                    Err(e) => { println!("Error: {}", e); }
                }
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
    // close the socket server
    drop(listener);
}