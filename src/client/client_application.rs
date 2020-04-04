 use std::net::{TcpStream, Ipv4Addr};
use mylib::common::properties::{Properties, get_properties};
use mylib::common::net::{confirm_distributed_barrier_client, DHTMessage, read_request_message_from_stream, get_key_from_dht_message};
use rand::Rng;
use rand::distributions::{Distribution, Uniform, Alphanumeric};
use mylib::common::net::DHTMessage::{Get, Put};
use mylib::common::my_hash;
use std::thread;
use std::io::Write;

// Generates and returns num_requests number of Get/Put requests randomly within the given key_range
fn generate_requests(num_requests: &u64, key_range: &Vec<u64>) -> Vec<DHTMessage> {
    let mut requests: Vec<DHTMessage> = Vec::new();
    let mut rng = rand::thread_rng();
    let mut keys: Vec<String> = Vec::new();
    for _ in key_range[0]..key_range[1] { keys.push(rng.sample_iter(&Alphanumeric).take(10).collect()); }
    let request_type_range = Uniform::from(0..5);
    let key_range_distribution = Uniform::from(key_range[0]..(key_range[1]));
    println!("Generating requests!");
    for _ in 0..*num_requests {
        let index= key_range_distribution.sample(&mut rng);
        let key = keys[index as usize].clone();
        match request_type_range.sample(&mut rng) {
            0 | 1 | 2 => { requests.push(Get(key)); } //Get
            _ => { requests.push(Put(key, rng.sample_iter(&Alphanumeric).take(30).collect())); } //Put
        }
    }
    return requests;
}

//Makes connections and gets the stream objects for all the servers in the same order as the node_ips
fn get_server_streams(node_ips: &Vec<Ipv4Addr>, server_port: &u64) -> Vec<TcpStream> {
   let mut streams: Vec<TcpStream> = Vec::new();
   for ip in node_ips {
       loop {
           match TcpStream::connect(ip.to_string() + ":" + &server_port.to_string()) {
               Ok(stream) => {
                   streams.push(stream);
                   break;
               }
               Err(e) => { println!("Failed to connect: {}. Retrying...", e); }
           }
       }
   }
   return streams;
}

// fn get_replicated_nodes(key: &u64) -> Vec<u64> {
//
// }

// Sends the requests to the appropriate server(s) one by one
fn send_requests(mut requests: Vec<DHTMessage>, mut streams: Vec<TcpStream>) {
    let num_nodes = streams.len();
    while !requests.is_empty() {
        let request = requests.pop().unwrap();
        // We add a random salt string because the same hash is used in the hashtable, so we don't want the same mappings of keys to nodes and buckets (bad performance)
        let which_node: usize = my_hash(get_key_from_dht_message(&request) + "random salt!") as usize % num_nodes; //mods the key by the number of nodes

        // send request
        &streams[which_node].write_all(bincode::serialize(&request).unwrap().as_slice());

        // wait for and receive response from server
        match read_request_message_from_stream(&streams[which_node]) {
            Ok(response) => {
                //println!("Handling response on client");
                match response {
                    DHTMessage::RequestFailed => {
                        // Got a negative response, so we try the same request again
                        requests.push(request);
                    }
                    DHTMessage::GetResponse(_option) => {}
                    DHTMessage::PutResponse(_success) => {}
                    _ => { panic!("Unexpected response from server. Expected DHTMessage::Response"); }
                }

            }
            Err(e) => { panic!("Error reading response: {}", e); }
        }
    }
}

//handles client application work
fn client_process(thread_num: &usize, properties: &Properties) {
    // Generate num_requests number of requests randomly
    let requests = generate_requests(&properties.num_requests, &properties.key_range);

    // Establish persistent connections with all the servers
    let streams = get_server_streams(&properties.node_ips, &properties.server_port);

    // Make requests to the appropriate server
    println!("Sending requests for client thread {}...", thread_num);
    send_requests(requests, streams);
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
        join_handle.join().expect("Error join handle.");
    }
}