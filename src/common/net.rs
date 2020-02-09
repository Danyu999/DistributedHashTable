use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::error::Error;
use serde::{Serialize, Deserialize};
use crate::common::net::BarrierMessage::{OneReady, AllReady};
use std::thread;
use std::sync::Arc;
use crate::common::net::DHTMessage::{Get, Put};

#[derive(Serialize, Deserialize)]
pub enum BarrierMessage {
    AllReady,
    OneReady,
}

#[derive(Serialize, Deserialize)]
pub enum DHTMessage {
    Get(u64), //(key)
    Put(u64, String), //(key, sizeOfContent, content)
    GetResponse(Option<String>),
    PutResponse(bool),
    RequestFailed,
}

pub fn get_key_from_dht_message(msg: &DHTMessage) -> u64 {
    match msg {
        Get(key) => { *key },
        Put(key, _) => { *key },
        _ => { panic!("expected Get or Put type message") }
    }
}

//from https://docs.serde.rs/serde_json/de/fn.from_reader.html
pub fn read_barrier_message_from_stream(stream: &TcpStream) -> Result<BarrierMessage, Box<dyn Error>> {
    let mut de = serde_json::Deserializer::from_reader(stream);
    let req = BarrierMessage::deserialize(&mut de)?;

    Ok(req)
}

pub fn read_request_message_from_stream(stream: &TcpStream) -> Result<DHTMessage, Box<dyn Error>> {
    let mut de = serde_json::Deserializer::from_reader(stream);
    let req = DHTMessage::deserialize(&mut de)?;

    Ok(req)
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
        //server_port will represent the server, while server_port+1 will be the temp client port
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
pub fn confirm_distributed_barrier(server_port: &u64, node_ips: &Vec<Ipv4Addr>, is_server: bool) -> bool {
    //listen and count the number of ready messages received
    let port: u64;
    if is_server { port = *server_port; } else { port = server_port + 1; } //differentiate between clients and servers
    let sync_port = Arc::new(port.clone());
    let sync_node_ips = Arc::new(node_ips.clone());
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port.to_string()).unwrap();
    println!("Process listening for barrier msgs on port {}", &port);
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
                                    println!("Received everybody is ready!");
                                    let server_port_copy = Arc::clone(&sync_port);
                                    let node_ips_copy = Arc::clone(&sync_node_ips);
                                    println!("Broadcasting AllReady to all processes");
                                    thread::spawn(move || { broadcast_all_barrier_message(&server_port_copy, &node_ips_copy, BarrierMessage::AllReady) });
                                }
                            }
                            AllReady => {
                                println!("AllReady message received");
                                num_all_ready += 1;
                                if num_all_ready == sync_node_ips.len() {
                                    println!("Received everybody is all ready!");
                                    return true;
                                }
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