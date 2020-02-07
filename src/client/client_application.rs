use std::net::{TcpStream, Ipv4Addr};
use mylib::common::properties::{Properties, get_properties};
use mylib::common::net::{confirm_distributed_barrier, DHTMessage, read_request_message_from_stream, get_key_from_dht_message};
use rand::Rng;
use rand::distributions::{Distribution, Uniform, Alphanumeric};
use mylib::common::net::DHTMessage::{Get, Put};

fn generate_requests(num_requests: &u64, key_range: &Vec<u64>) -> Vec<DHTMessage> {
    let mut requests: Vec<DHTMessage> = Vec::new();
    let mut rng = rand::thread_rng();
    let request_type_range = Uniform::from(0..2);
    let key_range_distribution = Uniform::from(key_range[0]..key_range[1]);
    println!("Generating requests!");
    for _ in 0..*num_requests {
        let key = key_range_distribution.sample(&mut rng);
        match request_type_range.sample(&mut rng) {
            0 => { requests.push(Get(key)); } //Get
            _ => { requests.push(Put(key, rng.sample_iter(&Alphanumeric).take(30).collect())); } //Put
        }
    }
    return requests;
}


////Currently unused. Can be useful if the server has a thread dedicated to each client connection.
//fn get_server_streams(mut node_ips: Vec<Ipv4Addr>, server_port: u64) -> Vec<TcpStream>{
//    let mut streams: Vec<TcpStream> = Vec::new();
//    while !node_ips.is_empty() {
//        let ip = node_ips.pop().unwrap();
//        match TcpStream::connect(ip.to_string() + &server_port.to_string()) {
//            Ok(stream) => {
//                streams.push(stream);
//            }
//            Err(e) => {
//                println!("Failed to connect: {}. Retrying...", e);
//                node_ips.push(ip);
//            }
//        }
//    }
//    return streams;
//}

fn send_requests(mut requests: Vec<DHTMessage>, node_ips: Vec<Ipv4Addr>, server_port: u64) -> bool {
    //TODO: create streams for all nodes, instead of remaking connections all the time (is this a good idea?)
    let num_nodes = node_ips.len();
    //let node_streams = get_server_streams(node_ips, server_port);
    while !requests.is_empty() {
        let request = requests.pop().unwrap();
        let which_node: usize = get_key_from_dht_message(&request) as usize % num_nodes; //mods the key by the number of nodes

        loop { //keeps retrying until the message goes through successfully
            println!("Connecting to {}", node_ips[which_node].to_string() + ":" + &server_port.to_string());
            match TcpStream::connect(node_ips[which_node].to_string() + ":" + &server_port.to_string()) {
                Ok(stream) => {
                    // send request
                    //serde_json::to_writer(&node_streams[which_node], &request).unwrap();
                    serde_json::to_writer(&stream, &request).unwrap();

                    // wait for and receive response from server
                    match read_request_message_from_stream(&stream/*&node_streams[which_node]*/) {
                        Ok(response) => {
                            match response {
                                DHTMessage::Response(success) => {
                                    if !success {
                                        // Got a negative response, so we try the same request again
                                        requests.push(request);
                                    }
                                }
                                _ => { println!("Unexpected response from server. Expected DHTMessage::Response"); }
                            }

                        }
                        Err(e) => { println!("Error reading response: {}", e); return false; }
                    }

                    break;
                }
                Err(e) => { println!("Failed to connect: {}. Retrying...", e); }
            }
        }
    }
    return true;
}

fn main() {
    let properties: Properties = get_properties();
    if !confirm_distributed_barrier(&properties.server_port, &properties.node_ips, false) {
        panic!("Distributed barrier for client failed!");
    }

    // Generate num_requests number of requests randomly
    let requests = generate_requests(&properties.num_requests, &properties.key_range);

    // Make requests to the appropriate server
    println!("Sending requests...");
    send_requests(requests, properties.node_ips, properties.server_port);
    println!("Client terminated.");
}