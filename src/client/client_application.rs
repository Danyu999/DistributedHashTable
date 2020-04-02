use std::net::{TcpStream, Ipv4Addr};
use mylib::common::properties::{Properties, get_properties};
use mylib::common::net::{confirm_distributed_barrier_client, DHTMessage, read_request_message_from_stream, get_key_from_dht_message};
use rand::Rng;
use rand::distributions::{Distribution, Uniform, Alphanumeric};
use mylib::common::net::DHTMessage::{Get, Put};
use mylib::common::metrics::Metrics;
use std::time::Instant;
use mylib::common::my_hash;
use std::thread;

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


//Makes connections and gets the stream objects for all the servers in the same order as the node_ips
fn get_server_streams(node_ips: &Vec<Ipv4Addr>, server_port: &u64) -> Vec<TcpStream>{
   let mut streams: Vec<TcpStream> = Vec::new();
   for ip in node_ips {
       loop {
           match TcpStream::connect(ip.to_string() + ":" + &server_port.to_string()) {
               Ok(stream) => {
                   streams.push(stream);
                   break;
               }
               Err(e) => {
                   println!("Failed to connect: {}. Retrying...", e);
               }
           }
       }
   }
   return streams;
}

fn get_replicated_nodes(key: &u64) {}

// Sends the requests to the appropriate server(s) one by one
fn send_requests(mut requests: Vec<DHTMessage>, streams: Vec<TcpStream>, metrics: &mut Metrics) {
    let start = Instant::now();
    let mut start_operation;
    //TODO: keep persistent connections/streams to each server; never close a stream
    let num_nodes = streams.len();
    while !requests.is_empty() {
        let request = requests.pop().unwrap();
        let which_node: usize = my_hash(get_key_from_dht_message(&request)) as usize % num_nodes; //mods the key by the number of nodes

        // send request
        start_operation = Instant::now();
        serde_json::to_writer(&streams[which_node], &request).unwrap();

        // wait for and receive response from server
        match read_request_message_from_stream(&streams[which_node]) {
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
                    _ => { panic!("Unexpected response from server. Expected DHTMessage::Response"); }
                }

            }
            Err(e) => { panic!("Error reading response: {}", e); }
        }

        metrics.time_one_operation.push(start_operation.elapsed().as_micros());
    }
    metrics.total_time = start.elapsed().as_micros();
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
    println!("Some gets: {}", metrics.some_get);
    println!("None gets: {}", metrics.none_get);
    println!("Failed requests: {}", metrics.failed_request);
    println!("Total send_requests time: {} seconds", metrics.total_time as f64/1000000 as f64);
    println!("Average number of operations per second: {}", metrics.num_operations as f64/(metrics.total_time as f64/1000000 as f64));
    println!("Average time for system to accomplish one operation: {} milliseconds", mean(&metrics.time_one_operation)/1000 as f64);
}

//handles client application work
fn client_process(thread_num: &usize, properties: &Properties) {
    // Generate num_requests number of requests randomly
    let requests = generate_requests(&properties.num_requests, &properties.key_range);

    //TODO: move to server-side
    let mut metrics = Metrics::new();
    metrics.key_range_size = properties.key_range[1] - properties.key_range[0];
    metrics.num_operations = properties.num_requests;

    let streams = get_server_streams(&properties.node_ips, &properties.server_port);

    // Make requests to the appropriate server
    println!("Sending requests for client thread {}...", thread_num);
    send_requests(requests, streams, &mut metrics);
    print_metrics(metrics);
    println!("Client thread {} terminated.", thread_num);
}

fn main() {
    let properties: Properties = get_properties();

    // Does the distributed barrier, ensuring all servers are up and ready before continuing
    confirm_distributed_barrier_client(&properties.server_client_check_port, &properties.node_ips);

    // Spawns num_client_threads number of threads to make client requests. The main thread then waits for all spawned threads to finish
    let mut join_handles: Vec<thread::JoinHandle<_>> = Vec::new();
    for i in 0..properties.num_client_threads {
        let properties_copy = properties.clone();
        let thread_num = i.clone();
        join_handles.push(thread::spawn(move || { client_process(&thread_num, &properties_copy) }) );
    }

    for join_handle in join_handles {
        // println!("Main thread joining...");
        join_handle.join();
    }
}