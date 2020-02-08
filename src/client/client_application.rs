use std::net::{TcpStream, Ipv4Addr};
use mylib::common::properties::{Properties, get_properties};
use mylib::common::net::{confirm_distributed_barrier, DHTMessage, read_request_message_from_stream, get_key_from_dht_message};
use rand::Rng;
use rand::distributions::{Distribution, Uniform, Alphanumeric};
use mylib::common::net::DHTMessage::{Get, Put};
use mylib::common::metrics::Metrics;
use std::time::Instant;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// Hash function to hash the keys. Each DefaultHasher created with new is guaranteed to be the same as others
fn my_hash<T>(obj: T) -> u64 where T: Hash, {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

// Generates and returns num_requests number of Get/Put requests randomly within the given key_range
fn generate_requests(num_requests: &u64, key_range: &Vec<u64>) -> Vec<DHTMessage> {
    let mut requests: Vec<DHTMessage> = Vec::new();
    let mut rng = rand::thread_rng();
    let request_type_range = Uniform::from(0..5);
    let key_range_distribution = Uniform::from(key_range[0]..(key_range[1]));
    println!("Generating requests!");
    for _ in 0..*num_requests {
        let key = key_range_distribution.sample(&mut rng);
        match request_type_range.sample(&mut rng) {
            0 | 1 | 2 => { requests.push(Get(key)); } //Get
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

// Sends the requests to the appropriate server one by one
fn send_requests(mut requests: Vec<DHTMessage>, node_ips: Vec<Ipv4Addr>, server_port: u64, metrics: &mut Metrics) -> bool {
    let start = Instant::now();
    let mut start_operation;
    //TODO: create streams for all nodes, instead of remaking connections all the time (is this a good idea?)
    let num_nodes = node_ips.len();
    //let node_streams = get_server_streams(node_ips, server_port);
    while !requests.is_empty() {
        let request = requests.pop().unwrap();
        let which_node: usize = my_hash(get_key_from_dht_message(&request)) as usize % num_nodes; //mods the key by the number of nodes

        loop { //keeps retrying until the message goes through successfully
            //println!("Connecting to {}", node_ips[which_node].to_string() + ":" + &server_port.to_string());
            match TcpStream::connect(node_ips[which_node].to_string() + ":" + &server_port.to_string()) {
                Ok(stream) => {
                    // send request
                    //serde_json::to_writer(&node_streams[which_node], &request).unwrap();
                    start_operation = Instant::now();
                    serde_json::to_writer(&stream, &request).unwrap();

                    // wait for and receive response from server
                    match read_request_message_from_stream(&stream/*&node_streams[which_node]*/) {
                        Ok(response) => {
                            match response {
                                DHTMessage::RequestFailed => {
                                    // Got a negative response, so we try the same request again
                                    metrics.failed_request += 1;
                                    requests.push(request);
                                }
                                DHTMessage::GetResponse(option) => {
                                    if option.is_some() {
                                        metrics.some_get += 1;
                                    } else {
                                        metrics.none_get += 1;
                                    }
                                }
                                DHTMessage::PutResponse(success) => {
                                    if success {
                                        metrics.successful_put += 1;
                                    } else {
                                        metrics.unsuccessful_put += 1;
                                    }
                                }
                                _ => { println!("Unexpected response from server. Expected DHTMessage::Response"); return false; }
                            }

                        }
                        Err(e) => { println!("Error reading response: {}", e); return false; }
                    }

                    break;
                }
                Err(_) => { } //Normally would print failed to connect, retrying, but repetitive I/O like that would slow down the program a lot
            }
        }
        metrics.time_one_operation.push(start_operation.elapsed().as_micros());
    }
    metrics.total_time = start.elapsed().as_micros();
    return true;
}

fn mean(list: &Vec<u128>) -> f64 {
    let mut sum: u128 = 0;
    for ele in list.iter() { sum += *ele; }
    sum as f64 / list.len() as f64
}

// Prints out the collected metrics
// TODO: write results to a file
fn print_metrics(metrics: Metrics) {
    println!("Total number of operations: {}", metrics.num_operations);
    println!("Key range size: {}", metrics.key_range_size);
    println!("Successful puts: {}", metrics.successful_put);
    println!("Unsuccessful puts: {}", metrics.unsuccessful_put);
    println!("Some get: {}", metrics.some_get);
    println!("None get: {}", metrics.none_get);
    println!("Failed requests: {}", metrics.failed_request);
    println!("Total send_requests time: {} seconds", metrics.total_time/1000000 as u128);
    println!("Average number of operations per second: {}", metrics.num_operations as f64/(metrics.total_time as f64/1000000 as f64));
    println!("Average time for system to accomplish one operation: {} seconds", mean(&metrics.time_one_operation)/1000000 as f64);
}

fn main() {
    let properties: Properties = get_properties();

    // Does the distributed barrier, ensuring all servers are up and ready before continuing
    // Note: Decided thread pool was overkill for distributed barrier, so it just spawns a few threads
    if !confirm_distributed_barrier(&properties.server_port, &properties.node_ips, false) {
        panic!("Distributed barrier for client failed!");
    }

    // Generate num_requests number of requests randomly
    let requests = generate_requests(&properties.num_requests, &properties.key_range);

    let mut metrics = Metrics::new();
    metrics.key_range_size = properties.key_range[1] - properties.key_range[0];
    metrics.num_operations = properties.num_requests;
    // Make requests to the appropriate server
    println!("Sending requests...");
    send_requests(requests, properties.node_ips, properties.server_port, &mut metrics);
    print_metrics(metrics);
    println!("Client terminated.");
}