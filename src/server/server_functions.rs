use std::net::{TcpStream, TcpListener};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream};
use std::sync::{Arc, Mutex};
use mylib::common::threadpool::ThreadPool;
use std::thread;
use std::io::Write;
use mylib::common::metrics::Metrics;
use std::time::Instant;
use std::sync::atomic::Ordering::Relaxed;
use mylib::common::my_hash;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

pub fn handle_client(mut stream: TcpStream, hashtable: Arc<Hashtable<String>>, lock_table: Arc<Vec<Mutex<bool>>>, metrics: Arc<Metrics>) {
    let mut start_operation: Instant;
    loop {
        match read_request_message_from_stream(&stream) {
            Ok(msg) => {
                let mut response: DHTMessage;
                start_operation = Instant::now();
                match msg {
                    DHTMessage::Get(key) => {
                        // Start phase one of 2PL
                        let bucket_index: usize = my_hash(key.as_str()) as usize % hashtable.num_buckets;
                        println!("CHECK: bucket_index_out: {}", bucket_index);
                        match lock_table[bucket_index].try_lock() {
                            // The lock is not taken, so we lock
                            Ok(_) => {
                                // Phase one locking is done, so we reply with an ack
                                response = DHTMessage::PhaseOneAck;
                                &stream.write_all(bincode::serialize(&response).unwrap().as_slice());

                                //Phase two starts, we listen for a commit or abort message
                                match read_request_message_from_stream(&stream) {
                                    Ok(msg) => {
                                        match msg {
                                            DHTMessage::Commit => {
                                                match hashtable.get(&key) {
                                                    Ok(val) => {
                                                        let val_clone = val.clone();
                                                        response = DHTMessage::GetResponse(val);
                                                        &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                                                        if val_clone == None { metrics.some_get.fetch_add(1, Relaxed); }
                                                        else { metrics.none_get.fetch_add(1, Relaxed); }
                                                        continue;
                                                    }
                                                    Err(_) => {}
                                                }
                                            }
                                            DHTMessage::Abort => {}
                                            _ => { panic!("Expected Commit or Abort request"); }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            // The bucket lock is taken, so the request fails
                            Err(_) => {}
                        }
                    }
                    DHTMessage::Put(key, val) => {
                        let bucket_index: usize = my_hash(key.as_str()) as usize % hashtable.num_buckets;
                        println!("CHECK: bucket_index_out: {}", bucket_index);
                        match lock_table[bucket_index].try_lock() {
                            // The lock is not taken, so we lock
                            Ok(_) => {
                                // Phase one locking is done, so we reply with an ack
                                response = DHTMessage::PhaseOneAck;
                                &stream.write_all(bincode::serialize(&response).unwrap().as_slice());

                                //Phase two starts, we listen for a commit or abort message
                                match read_request_message_from_stream(&stream) {
                                    Ok(msg) => {
                                        match msg {
                                            DHTMessage::Commit => {
                                                match hashtable.insert(key, val) {
                                                    Ok(ret) => {
                                                        let ret_clone = ret.clone();
                                                        response = DHTMessage::PutResponse(ret);
                                                        &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                                                        if ret_clone { metrics.put_insert.fetch_add(1, Relaxed); } else { metrics.put_update.fetch_add(1, Relaxed); }
                                                    }
                                                    Err(_) => {}
                                                }
                                            }
                                            DHTMessage::Abort => {}
                                            _ => { panic!("Expected Commit or Abort request"); }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            // The bucket lock is taken, so the request fails
                            Err(_) => { }
                        }
                    }
                    _ => { panic!("Expected Get or Put request"); }
                }
                response = DHTMessage::RequestFailed;
                &stream.write_all(bincode::serialize(&response).unwrap().as_slice());
                metrics.failed_request.fetch_add(1, Relaxed);
                metrics.total_time_operations.fetch_add(start_operation.elapsed().as_micros(), Relaxed);
                metrics.num_operations.fetch_add(1, Relaxed);
            }
            Err(e) => { println!("Connection possibly closed: {}", e); return; }
        }
    }
}

pub fn accept_client(port: &u64, hashtable: &mut Arc<Hashtable<String>>, _pool: &ThreadPool, metrics: Arc<Metrics>) {
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port.to_string()).unwrap();
    let lock_table : Arc<Vec<Mutex<bool>>> = Arc::new(Vec::new());

    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port {}", &port);
    for stream in listener.incoming() {
        //println!("Connection accepted! Handling...");
        match stream {
            Ok(stream) => {
                let hashtable_clone = hashtable.clone();
                let metrics_clone = metrics.clone();
                let lock_table_clone = lock_table.clone();
                // pool.execute(|| { handle_client(stream, hashtable_clone) });
                thread::spawn(move || { handle_client(stream, hashtable_clone, lock_table_clone, metrics_clone) });
                //println!("Done handling, listening again...")
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
}