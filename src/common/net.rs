use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::error::Error;
use serde::{Serialize, Deserialize, Serializer};
use crate::common::net::BarrierMessage::{OneReady, AllReady};

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
    for node in node_ips {
        //server_port will represent the server, while server_port+1 will be the temp client port
        for i in 0..2 {
            match TcpStream::connect(node.to_string() + ":" + &(server_port + i).to_string()) {
                Ok(stream) => {
                    println!("Connected to {}!", server_port + i);
                    serde_json::to_writer(&stream, &msg).unwrap();
                }
                Err(e) => { println!("Error: {}", e); return false; }
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
    //listen and count the number of ready messages received
    let port: String;
    if is_server { port = server_port.to_string() } else { port = (server_port + 1).to_string()}
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port).unwrap();
    println!("Process listening on port {}", &port);
    //send out ready messages to all processes on the network
    if !broadcast_all_barrier_message(server_port, node_ips, BarrierMessage::OneReady) {
        return false;
    }
    let mut num_ready = 0;
    let mut num_all_ready = 0;
    while num_all_ready != node_ips.len() {
        //if we have received a ready from each process in the network, we send an all ready signal
        if num_ready == node_ips.len() {
            if !broadcast_all_barrier_message(server_port, node_ips, BarrierMessage::AllReady) {
                return false;
            }
        }
        //accept a new connection
        let connection = listener.accept();
        match connection {
            Ok(stream) => {
                match read_barrier_message_from_stream(&stream.0) {
                    Ok(msg) => {
                        match msg {
                            OneReady => {
                                println!("OneReady message received");
                                num_ready += 1;
                            }
                            AllReady => {
                                println!("AllReady message received");
                                num_all_ready += 1;
                            }
                        }
                    }
                    Err(e) => { println!("Error reading message from stream! {}", e); return false; }
                }
            }
            Err(e) => { println!("Error connecting to a process: {}", e); return false; }
        }
    }
    return true;
}