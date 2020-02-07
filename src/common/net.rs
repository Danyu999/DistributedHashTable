use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::error::Error;
use serde::{Serialize, Deserialize, Serializer};
use crate::common::net::BarrierMessage::{OneReady, AllReady};
use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub enum BarrierMessage {
    AllReady,
    OneReady,
}

#[derive(Serialize, Deserialize)]
pub enum RequestMessage {
    Get(u64), //(key)
    Put(u64, u64, String), //(key, sizeOfContent, content)
}

/*impl Serialize for BarrierMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("BarrierMessage", 2)?;
        state.serialize_tuple_variant("AllReady", &self.AllReady)?;
        state.serialize_tuple_variant("OneReady", &self.OneReady)?;
        state.end()
    }
}*/

//from https://docs.serde.rs/serde_json/de/fn.from_reader.html
pub fn read_barrier_message_from_stream(stream: &TcpStream) -> Result<BarrierMessage, Box<dyn Error>> {
    let mut de = serde_json::Deserializer::from_reader(stream);
    let req = BarrierMessage::deserialize(&mut de)?;

    Ok(req)
}

pub fn read_request_message_from_stream(stream: &TcpStream) -> Result<RequestMessage, Box<dyn Error>> {
    let mut de = serde_json::Deserializer::from_reader(stream);
    let req = RequestMessage::deserialize(&mut de)?;

    Ok(req)
}


fn broadcast_all_barrier_message(server_port: &u64, node_ips: &Vec<Ipv4Addr>, msg: BarrierMessage) -> bool {
    println!("Broadcasting BarrierMessage with server_port {}", server_port);
    //connect to each node
    let mut node_ips_left = node_ips.clone();
    let mut i = 0;
    while !node_ips_left.is_empty() {
        if i == node_ips_left.len() {
            i = 0;
            continue;
        }
        println!("Trying to connect to: {}" , node_ips_left[i].to_string());
        //server_port will represent the server, while server_port+1 will be the temp client port
        for j in 0..1 {
            match TcpStream::connect(node_ips_left[i].to_string() + ":" + &(server_port + j).to_string()) {
                Ok(stream) => {
                    println!("Connected to {}!", server_port + j);
                    serde_json::to_writer(&stream, &msg).unwrap();
                    node_ips_left.swap_remove(i);
                }
                Err(e) => { println!("Error: {}", e); i += 1;}
            }
        }
    }
    println!("Done broadcasting!");
    return true;
}

/**
* Makes sure all processes, clients and servers, are up and running
* Returns true if everything is up and running, false if an error occurs
**/
pub fn confirm_distributed_barrier(server_port: &u64, node_ips: &Vec<Ipv4Addr>, is_server: bool) -> bool {
    let sync_server_port = Arc::new(server_port.clone());
    let sync_node_ips = Arc::new(node_ips.clone());

    //listen and count the number of ready messages received
    let port: String;
    if is_server { port = sync_server_port.to_string() } else { port = (server_port + 1).to_string()}
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port).unwrap();
    println!("Process listening on port {}", &port);
    let mut num_ready = 0;
    let mut num_all_ready = 0;

    //send out ready messages to all processes on the network
    let server_port_copy = Arc::clone(&sync_server_port);
    let node_ips_copy = Arc::clone(&sync_node_ips);
    thread::spawn(move || { broadcast_all_barrier_message(&server_port_copy, &node_ips_copy, BarrierMessage::OneReady) });

    //accept a new connection
    println!("Process {} listening for barrier messages...", &port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connected with sender!");
                match read_barrier_message_from_stream(&stream) {
                    Ok(msg) => {
                        match msg {
                            OneReady => {
                                println!("OneReady message received");
                                num_ready += 1;
                                //if we have received a ready from each process in the network, we send an all ready signal
                                if num_ready == sync_node_ips.len() {
                                    println!("Received everybody is ready!");
                                    let server_port_copy = Arc::clone(&sync_server_port);
                                    let node_ips_copy = Arc::clone(&sync_node_ips);
                                    thread::spawn(move || { broadcast_all_barrier_message(&server_port_copy, &node_ips_copy, BarrierMessage::AllReady) });
                                }
                            }
                            AllReady => {
                                println!("AllReady message received");
                                num_all_ready += 1;
                                if num_all_ready == sync_node_ips.len() {
                                    println!("Received everybody is all ready!");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading message from stream! {}", e);
                        return false;
                    }
                }
            }
            Err(e) => {
                println!("Error connecting to a process: {}", e);
                return false;
            }
        }
    }
    return true;
}