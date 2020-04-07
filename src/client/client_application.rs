 use std::net::{TcpStream, Ipv4Addr};
use mylib::common::properties::{Properties, get_properties};
use mylib::common::net::{confirm_distributed_barrier_client, DHTMessage, read_request_message_from_stream, get_key_from_dht_message, PutRequest, write_dht_message_to_stream};
use rand::Rng;
use rand::distributions::{Distribution, Uniform, Alphanumeric};
use mylib::common::net::DHTMessage::{Get, Put, MultiPut, GetResponse, RequestFailed, PhaseOneAck, Commit, Abort};
use mylib::common::my_hash;
use std::thread;
 use std::collections::HashMap;
 use std::thread::sleep;
 use std::time::Duration;
 use std::cmp::{min, max};

 // Generates and returns num_requests number of Get/Put requests randomly within the given key_range
fn generate_requests(num_requests: &u64, key_range: &Vec<usize>, multi_put_num: &usize) -> Vec<DHTMessage> {
    assert_eq!(2, key_range.len());
    let mut requests: Vec<DHTMessage> = Vec::new();
    let mut rng = rand::thread_rng();
    let request_type_range = Uniform::from(0..5);
    let key_range_distribution = Uniform::from(key_range[0]..(key_range[1]));
    // println!("Generating requests!");
    for _ in 0..*num_requests {

        match request_type_range.sample(&mut rng) {
            //Get
            0 | 1 | 2 => {
                let key= key_range_distribution.sample(&mut rng);
                requests.push(Get(key.to_string()));
            }
            //Put
            3 => {
                let key= key_range_distribution.sample(&mut rng);
                requests.push(Put(PutRequest {
                                            key: key.to_string(),
                                            val: rng.sample_iter(&Alphanumeric).take(30).collect()
                                        }));
            }
            //MultiPut
            4 | _ => {
                // If any of the keys are the same, that is acceptable. The execution order will be from left to right
                let mut multi_put : Vec<PutRequest> = Vec::new();
                for _ in 0..*multi_put_num {
                    let key= key_range_distribution.sample(&mut rng);
                    multi_put.push(PutRequest {
                        key: key.to_string(),
                        val: rng.sample_iter(&Alphanumeric).take(30).collect()
                    });
                }
                requests.push(MultiPut(multi_put));
            }
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
    assert_eq!(streams.len(), node_ips.len());
    return streams;
}

fn get_which_nodes(key: &String, num_nodes: &usize, replication_degree: &usize) -> Vec<usize> {
    // We add a random salt string because the same hash is used in the hashtable, so we don't want the same mappings of keys to nodes and buckets (bad performance)
    let mut nodes : Vec<usize> = Vec::new();
    if replication_degree >= num_nodes {
        for i in 0..*num_nodes {
            nodes.push(i);
        }
        return nodes;
    } else {
        let main_node = my_hash((key.to_string() + "random salt!").as_str()) as usize % *num_nodes; //mods the key by the number of nodes
        for i in main_node..(replication_degree + main_node) {
            if i >= *num_nodes {
                nodes.push(i - *num_nodes);
            } else {
                nodes.push(i);
            }
        }
        assert_eq!(*replication_degree, nodes.len());
        return nodes;
    }
}

//Exponential backoff with jitter
//Source: https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
 fn exponential_backoff(attempt: u32, workload_config: u128) {
    //The cap of how long we sleep for exponential backoff
    let cap : u128 = 15000000000; // 15 seconds
    let base : u128 = max(10000000, 125000000 - (25000000 * workload_config)); // 100 milliseconds for 10 key range, 50 milliseconds for 1000 key range
    let mut rng = rand::thread_rng();
    let mut top_range = cap;
    match u128::checked_pow(2, attempt) {
        Some(val1) => {
            match u128::checked_mul(base, val1) {
                Some(val2) => { top_range = val2; }
                None => {}
            }
        }
        None => {}
    }
    let sleep_time = rng.gen_range(0, min(cap, top_range));
    // println!("exp_backoff time: {}", sleep_time);
    sleep(Duration::new(0, sleep_time as u32));
}

// Sends the requests to the appropriate server(s) one by one
fn send_requests(mut requests: Vec<DHTMessage>, mut streams: Vec<TcpStream>, _num_client_threads: &usize, key_range: &usize, replication_degree: &usize) {
    let num_nodes = streams.len();
    // let workload_config = num_client_threads * num_nodes * requests.len() * replication_degree;
    let workload_config : u128 = (*key_range as f64).log10() as u128; //10 -> 1, 100 -> 2, 1000 -> 3, 10000 -> 4
    while !requests.is_empty() {
        // let start_operation: Instant = Instant::now();
        let request = requests.pop().unwrap();
        let mut success = false;
        // println!();

        // Handle making the request, depending on the type of request
        match request {
            Get(_) => {
                let key = get_key_from_dht_message(&request);
                let which_nodes = get_which_nodes(&key, &num_nodes, &replication_degree);
                let mut node_index : usize = 0;
                while !success {
                    if node_index == which_nodes.len() { node_index = 0; }

                    // send request
                    // let t1 = Instant::now();
                    write_dht_message_to_stream(&mut streams[which_nodes[node_index]], &request);
                    // println!("client get write: {}", t1.elapsed().as_micros());

                    // wait for and receive response from server
                    // let t2 = Instant::now();
                    match read_request_message_from_stream(&mut streams[which_nodes[node_index]]) {
                        Ok(response) => {
                            match response {
                                RequestFailed => { node_index += 1; }
                                GetResponse(_) => { success = true; }
                                _ => { panic!("Unexpected response from server. Expected DHTMessage::Response"); }
                            }
                        }
                        Err(e) => { panic!("Error reading response: {}", e); }
                    }
                    // println!("client get read: {}", t2.elapsed().as_micros());
                }
            }
            Put(_) => {
                let key = get_key_from_dht_message(&request);
                let which_nodes = get_which_nodes(&key, &num_nodes, &replication_degree);
                let mut attempt = 0;
                while !success {
                    attempt += 1;
                    // Start phase one and send a request
                    // let t1 = Instant::now();
                    for node in &which_nodes {
                        write_dht_message_to_stream(&mut streams[*node], &request);
                    }
                    // println!("client put write: {}", t1.elapsed().as_micros());

                    // Receive acks from all the servers, abort if at least one sends RequestFailed
                    // TODO: Reading serially instead of in parallel may be a performance slowdown
                    let mut acks: Vec<usize> = Vec::with_capacity(which_nodes.len());
                    // let t2 = Instant::now();
                    for node in &which_nodes {
                        // Wait for and receive response from server
                        // Note: Currently, we wait for responses from all servers before moving on, regardless of success or not
                        match read_request_message_from_stream(&mut streams[*node]) {
                            Ok(response) => {
                                match response {
                                    RequestFailed => {}
                                    PhaseOneAck => { acks.push(*node); }
                                    _ => { panic!("Phase one client error: Expected a RequestFailed or PhaseOneAck message."); }
                                }
                            }
                            Err(e) => { panic!("Error reading response: {}", e); }
                        }
                    }
                    // println!("client put read: {}", t2.elapsed().as_micros());

                    // Check the acks and start phase 2 if we received acks from all servers
                    // let t3 = Instant::now();
                    if acks.len() == which_nodes.len() {
                        // Send commit messages to all servers
                        for node in &which_nodes {
                            write_dht_message_to_stream(&mut streams[*node], &Commit);
                        }
                        for node in &which_nodes {
                            match read_request_message_from_stream(&mut streams[*node]) {
                                Ok(response) => {
                                    match response {
                                        Commit => {}
                                        _ => { panic!("Expected Commit messages from all servers for put!"); }
                                    }
                                }
                                Err(e) => { println!("Error reading commit from servers for put... {}", e); }
                            }
                        }
                        success = true;
                    } else {
                        // Send abort messages to all servers who responded with an ack
                        // Note: Server-side, the request aborts if the server had a RequestFailed, so no need to send an abort to said servers
                        for node in acks {
                            write_dht_message_to_stream(&mut streams[node], &Abort);
                        }
                        // we do an exponential backoff to reduce contention for locks
                        exponential_backoff(attempt, workload_config);
                    }
                    // println!("client write commit/abort: {}", t3.elapsed().as_micros());
                }
            }
            MultiPut(puts) => {
                // Send MultiPut to each server that only contains the Put commands relevant to that server
                let mut server_multi_puts : HashMap<usize, Vec<PutRequest>> = HashMap::new();
                for p in puts {
                    let which_nodes = get_which_nodes(&p.key, &num_nodes, &replication_degree);
                    for node in which_nodes {
                        if server_multi_puts.contains_key(&node) {
                            let s : &mut Vec<PutRequest> = server_multi_puts.get_mut(&node).unwrap();
                            s.push(PutRequest { key: p.key.clone(), val: p.val.clone() });
                        } else {
                            let mut s : Vec<PutRequest> = Vec::new();
                            s.push(PutRequest { key: p.key.clone(), val: p.val.clone() });
                            server_multi_puts.insert(node, s);
                        }
                    }
                }

                // Once we know which servers are needed and what is needed for each server, we send requests
                let mut attempt = 0;
                while !success {
                    attempt += 1;
                    // First send the request to all servers
                    for node in &server_multi_puts {
                        write_dht_message_to_stream(&mut streams[*node.0], &MultiPut(node.1.clone()));

                    }

                    // Receive acks from all the servers, abort if at least one sends RequestFailed
                    // TODO: Reading serially instead of in parallel may be a performance slowdown
                    let mut acks: Vec<usize> = Vec::with_capacity(server_multi_puts.len());
                    for i in &server_multi_puts {
                        // Wait for and receive response from server
                        // Note: Currently, we wait for responses from all servers before moving on, regardless of success or not
                        match read_request_message_from_stream(&mut streams[*i.0]) {
                            Ok(response) => {
                                match response {
                                    RequestFailed => {}
                                    PhaseOneAck => { acks.push(*i.0); }
                                    _ => { panic!("Phase one client error: Expected a RequestFailed or PhaseOneAck message."); }
                                }
                            }
                            Err(e) => { panic!("Error reading response: {}", e); }
                        }
                    }

                    // Check the acks and start phase 2 if we received acks from all servers
                    if acks.len() == server_multi_puts.len() {
                        // Send commit messages to all servers
                        for node in &server_multi_puts {
                            write_dht_message_to_stream(&mut streams[*node.0], &Commit);

                        }
                        for node in &server_multi_puts {
                            match read_request_message_from_stream(&mut streams[*node.0]) {
                                Ok(response) => {
                                    match response {
                                        Commit => {}
                                        _ => { panic!("Expected Commit messages from all servers for multiput!"); }
                                    }
                                }
                                Err(e) => { println!("Error reading commit from servers for multiput... {}", e); }
                            }
                        }
                        success = true;
                    } else {
                        // Send abort messages to all servers who responded with an ack
                        // Note: Server-side, the request aborts if the server had a RequestFailed, so no need to send an abort to said servers
                        for node in acks {
                            write_dht_message_to_stream(&mut streams[node], &Abort);
                        }
                        // we do an exponential backoff to reduce contention for locks
                        exponential_backoff(attempt, workload_config);
                    }
                }
            }
            _ => { panic ! ("Expected a Get, Put, or MultiPut request!"); }
        }
        // println!("Time 1 op: {}", start_operation.elapsed().as_micros());
        // println!();
    }
}

//handles client application work
fn client_process(thread_num: &usize, properties: &Properties) {
    // Generate num_requests number of requests randomly
    let requests = generate_requests(&properties.num_requests, &properties.key_range, &properties.multi_put_num);
    // let mut requests = Vec::new();
    // let mut multi_put : Vec<PutRequest> = Vec::new();
    // for i in 0..properties.multi_put_num {
    //     let key= i;
    //     multi_put.push(PutRequest {
    //         key: key.to_string(),
    //         val: String::from("test")
    //     });
    // }
    // requests.push(MultiPut(multi_put));

    // Establish persistent connections with all the servers
    let streams = get_server_streams(&properties.node_ips, &properties.server_port);

    // Make requests to the appropriate server
    println!("Sending requests for client thread {}...", thread_num);
    send_requests(requests, streams, &properties.num_client_threads, &(properties.key_range[1]-properties.key_range[0]), &properties.replication_degree);
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
    println!("Client application done...");
}