use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::error::Error;
use serde::{Serialize, Deserialize};
use crate::common::net::BarrierMessage::{OneReady, AllReady, ClientCheck};
use std::thread;
use std::sync::Arc;
use crate::common::net::DHTMessage::{Get, Put};

#[derive(Serialize, Deserialize)]
pub enum BarrierMessage {
    AllReady,
    OneReady,
    ClientCheck,
}

#[derive(Serialize, Deserialize)]
pub struct PutRequest {
    pub key: String,
    pub val: String
}

#[derive(Serialize, Deserialize)]
pub enum DHTMessage {
    Get(String), //(key)
    Put(PutRequest), //(key, content)
    MultiPut(Vec<PutRequest>),
    PhaseOneAck,
    Commit,
    Abort,
    GetResponse(Option<String>),
    PutResponse(bool),
    MultiPutResponse,
    RequestFailed,
}

pub fn get_key_from_dht_message(msg: &DHTMessage) -> String {
    match msg {
        Get(key) => {
            key.clone()
        },
        Put(p) => {
            p.key.clone()
        },
        _ => { panic!("expected Get or Put type message") }
    }
}

// Note: deserialize(&mut de)? blocks
// from https://docs.serde.rs/serde_json/de/fn.from_reader.html
pub fn read_barrier_message_from_stream(stream: &TcpStream) -> Result<BarrierMessage, Box<dyn Error>> {
    let mut de = serde_json::Deserializer::from_reader(stream);
    let req = BarrierMessage::deserialize(&mut de)?;

    Ok(req)
}

pub fn read_request_message_from_stream(stream: &TcpStream) -> Result<DHTMessage, Box<dyn Error>> {
    let x = bincode::deserialize_from(stream);
    match x {
        Ok(msg) => { Ok(msg) }
        Err(e) => { Err(e) }
    }
}


fn broadcast_all_barrier_message(server_port: &u64, node_ips: &Vec<Ipv4Addr>, msg: BarrierMessage) -> bool {
    //connect to each node
    let mut node_ips_left = node_ips.clone();
    let mut i = 0;
    while !node_ips_left.is_empty() {
        if i == node_ips_left.len() {
            i = 0;
            continue;
        }
        //println!("Trying to connect to: {}" , node_ips_left[i].to_string());
        match TcpStream::connect(node_ips_left[i].to_string() + ":" + &server_port.to_string()) {
            Ok(stream) => {
                //println!("Connected to {}!", server_port);
                serde_json::to_writer(&stream, &msg).unwrap();
                node_ips_left.swap_remove(i);
            }
            Err(_) => { i += 1;}
        }
    }
    return true;
}

/**
* Makes sure all processes are up and running
* Returns true if everything is up and running, false if an error occurs
**/
pub fn confirm_distributed_barrier_server(server_port: &u64, node_ips: &Vec<Ipv4Addr>) -> bool {
    //listen and count the number of ready messages received
    let sync_port = Arc::new(server_port.clone());
    let sync_node_ips = Arc::new(node_ips.clone());
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &server_port.to_string()).unwrap();
    println!("Process listening for barrier msgs on port {}", &server_port);
    let mut num_ready = 0;
    let mut num_all_ready = 0;

    //send out ready messages to all processes on the network
    let server_port_copy = Arc::clone(&sync_port);
    let node_ips_copy = Arc::clone(&sync_node_ips);
    println!("Broadcasting OneReady to all processes");
    thread::spawn(move || { broadcast_all_barrier_message(&server_port_copy, &node_ips_copy, BarrierMessage::OneReady) });

    //accept a new connection
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                match read_barrier_message_from_stream(&stream) {
                    Ok(msg) => {
                        match msg {
                            OneReady => {
                                println!("OneReady message received");
                                num_ready += 1;
                                //if we have received a ready from each process in the network, we send an all ready signal
                                if num_ready == sync_node_ips.len() {
                                    //println!("Received everybody is ready!");
                                    let server_port_copy = Arc::clone(&sync_port);
                                    let node_ips_copy = Arc::clone(&sync_node_ips);
                                    //println!("Broadcasting AllReady to all processes");
                                    thread::spawn(move || { broadcast_all_barrier_message(&server_port_copy, &node_ips_copy, BarrierMessage::AllReady) });
                                }
                            }
                            AllReady => {
                                //println!("AllReady message received");
                                num_all_ready += 1;
                                if num_all_ready == sync_node_ips.len() {
                                    println!("Received everybody is all ready!");
                                    return true;
                                }
                            }
                            _ => {
                                println!("Unexpected message received!");
                                return false;
                            }
                        }
                    }
                    Err(e) => { println!("Error reading message from stream! {}", e); return false; }
                }
            }
            Err(e) => { println!("Error connecting to a process: {}", e); return false; }
        }
    }
    return false;
}

// An always spinning thread on all servers that confirms any asking clients that it is up and ready.
// Only spins after server is up and running
pub fn handle_client_checks(port: &u64) {
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port.to_string()).unwrap();
    //println!("Process listening for client check barrier msgs on port {}", &port);
    //accept a new connection
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                match read_barrier_message_from_stream(&stream) {
                    Ok(msg) => {
                        match msg {
                            ClientCheck => {
                                //barrier msg from a client. respond with OneReady msg
                                println!("ClientCheck message received");
                                thread::spawn(move || { serde_json::to_writer(&stream, &BarrierMessage::OneReady).unwrap() });
                            }
                            _ => {
                                println!("Unexpected message received!");
                            }
                        }
                    }
                    Err(e) => { println!("Error reading message from stream! {}", e); }
                }
            }
            Err(e) => { println!("Error connecting to a process: {}", e); }
        }
    }
}

// Creates a new connection with each server, checking if they are up
// Doesn't return until it confirms all servers are up
pub fn confirm_distributed_barrier_client(server_port: &u64, node_ips: &Vec<Ipv4Addr>) {
    let mut node_ips_left = node_ips.clone();
    let mut i = 0;
    let msg = ClientCheck;
    while !node_ips_left.is_empty() {
        if i == node_ips_left.len() {
            i = 0;
            continue;
        }
        match TcpStream::connect(node_ips_left[i].to_string() + ":" + &server_port.to_string()) {
            Ok(stream) => {
                //println!("Connected to {} for client barrier!", server_port);
                serde_json::to_writer(&stream, &msg).unwrap();
                match read_barrier_message_from_stream(&stream) {
                    Ok(msg) => {
                        match msg {
                            OneReady => {
                                //println!("Server ack received!");
                                node_ips_left.swap_remove(i);
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    }
                    Err(_) => { i += 1; }
                }
            }
            Err(_) => { i += 1; }
        }
    }
}