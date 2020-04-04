use std::net::{TcpStream, TcpListener};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream};
use std::sync::Arc;
use mylib::common::threadpool::ThreadPool;
use std::thread;
use std::io::Write;
use mylib::common::metrics::Metrics;
use std::time::Instant;
use std::sync::atomic::Ordering::Relaxed;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

pub fn handle_client(mut stream: TcpStream, hashtable: Arc<Hashtable<String>>, mut metrics: Arc<Metrics>) {
    let mut start_operation: Instant;
    loop {
        match read_request_message_from_stream(&stream) {
            Ok(msg) => {
                let mut response = DHTMessage::RequestFailed;
                start_operation = Instant::now();
                match msg {
                    DHTMessage::Get(key) => {
                        match hashtable.get(&key) {
                            Ok(val) => {
                                let val_clone = val.clone();
                                response = DHTMessage::GetResponse(val);
                                &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                                if val_clone == None { metrics.some_get.fetch_add(1, Relaxed); }
                                else { metrics.none_get.fetch_add(1, Relaxed); }
                            }
                            Err(_) => {
                                response = DHTMessage::RequestFailed;
                                &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                                metrics.failed_request.fetch_add(1, Relaxed);
                            }
                        }
                    }
                    DHTMessage::Put(key, val) => {
                        match hashtable.insert(key, val) {
                            Ok(success) => {
                                response = DHTMessage::PutResponse(success);
                                &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                                if success { metrics.successful_put.fetch_add(1, Relaxed); }
                                else { metrics.unsuccessful_put.fetch_add(1, Relaxed); }
                            }
                            Err(_) => {
                                response = DHTMessage::RequestFailed;
                                &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                                metrics.failed_request.fetch_add(1, Relaxed);
                            }
                        }
                    }
                    _ => {
                        println!("Expected Get or Put request");
                        &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                    }
                }
                metrics.total_time_operations.fetch_add(start_operation.elapsed().as_micros(), Relaxed);
                metrics.num_operations.fetch_add(1, Relaxed);
            }
            Err(e) => { println!("Connection possible closed: {}", e); return; }
        }
    }
}

pub fn accept_client(port: &u64, hashtable: &mut Arc<Hashtable<String>>, _pool: &ThreadPool, metrics: Arc<Metrics>) {
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port.to_string()).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port {}", &port);
    for stream in listener.incoming() {
        //println!("Connection accepted! Handling...");
        match stream {
            Ok(stream) => {
                let hashtable_clone = hashtable.clone();
                let metrics_clone = metrics.clone();
                // pool.execute(|| { handle_client(stream, hashtable_clone) });
                thread::spawn(move || { handle_client(stream, hashtable_clone, metrics_clone) });
                //println!("Done handling, listening again...")
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
}