use std::net::{TcpStream, TcpListener};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream, PutRequest};
use std::sync::Arc;
use mylib::common::threadpool::ThreadPool;
use std::thread;
use std::io::Write;
use mylib::common::metrics::Metrics;
use std::time::Instant;
use std::sync::atomic::Ordering::Relaxed;
use mylib::common::my_hash;
use mylib::common::net::DHTMessage::{PhaseOneAck, GetResponse, Commit, Abort, RequestFailed, Get, Put, MultiPut};
use mylib::common::locktable::Locktable;
use std::collections::HashSet;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

// TODO: Possible implementation idea for MultiPut where we recursively acquire the locks; This method would allow MultiPut to have
// returns Commit if successful, Abort if aborted, and RequestFailed if failed (3 different behaviors for metrics)
// This method does all the stream writing and reading
fn multi_put_helper(stream: &mut TcpStream, hashtable: Arc<Hashtable<String>>, lock_table: Arc<Locktable>, num_buckets: &usize,
                    puts: Vec<PutRequest>, lock_progress: usize, mut prev_buckets: HashSet<usize>) -> DHTMessage {
    // We recursively try to acquire the locks for each put operation, then continues the 2PL while holding all locks
    // println!("Entering multi_put_helper {}...", lock_progress);
    return if lock_progress == puts.len() {
        // We continue the operation since we have all the locks
        // Phase one locking is done, so we reply with an ack
        &stream.write_all(bincode::serialize(&PhaseOneAck).unwrap().as_slice());
        match read_request_message_from_stream(&stream) {
            Ok(msg) => {
                match msg {
                    Commit => {
                        for p in puts {
                            match hashtable.insert(p.key, p.val) {
                                Ok(_) => {} //As long as it succeeds, we don't care whether the put inserted or updated
                                Err(_) => { panic!("Lock acquired, but failed to get lock in hashtable!"); }
                            }
                        }
                        Commit
                    }
                    Abort => { Abort }
                    _ => { panic!("Expected Commit or Abort request"); }
                }
            }
            Err(e) => { panic!("Error reading message from stream: {}", e.to_string()); }
        }
    } else {
        // We continue acquiring locks
        let bucket_index: usize = my_hash(puts[lock_progress].key.as_str()) as usize % *num_buckets;
        if prev_buckets.contains(&bucket_index) {
            multi_put_helper(stream, hashtable.clone(), lock_table.clone(), &num_buckets, puts,
                             lock_progress + 1, prev_buckets)
        } else {
            // Lock not acquired yet, so we try to acquire
            match lock_table.locks[bucket_index].try_lock() {
                Ok(_) => {
                    prev_buckets.insert(bucket_index);
                    multi_put_helper(stream, hashtable.clone(), lock_table.clone(), &num_buckets, puts,
                                     lock_progress + 1, prev_buckets)
                }
                Err(_) => { RequestFailed }
            }
        }
    }
}

pub fn handle_client(mut stream: TcpStream, hashtable: Arc<Hashtable<String>>, lock_table: Arc<Locktable>, metrics: Arc<Metrics>) {
    let mut start_operation: Instant;
    loop {
        match read_request_message_from_stream(&stream) {
            Ok(msg) => {
                start_operation = Instant::now();
                match msg {
                    Get(key) => {
                        // Get request, so we don't do 2PL
                        let bucket_index: usize = my_hash(key.as_str()) as usize % hashtable.num_buckets;
                        match lock_table.locks[bucket_index].try_lock() {
                            // The lock is not taken, so we lock
                            Ok(_) => {
                                match hashtable.get(&key) {
                                    Ok(val) => {
                                        let val_clone = val.clone();
                                        &stream.write_all(bincode::serialize(&GetResponse(val)).unwrap().as_slice());
                                        if val_clone == None { metrics.some_get.fetch_add(1, Relaxed); }
                                        else { metrics.none_get.fetch_add(1, Relaxed); }
                                    }
                                    Err(_) => { panic!("Lock acquired for get, but failed to get lock in hashtable!"); }
                                }
                            }
                            Err(_) => {
                                &stream.write_all(bincode::serialize(&RequestFailed).unwrap().as_slice());
                                metrics.failed_request.fetch_add(1, Relaxed);
                            }
                        }
                    }
                    Put(p) => {
                        let bucket_index: usize = my_hash(p.key.as_str()) as usize % hashtable.num_buckets;
                        match lock_table.locks[bucket_index].try_lock() {
                            // The lock is not taken, so we lock
                            Ok(_) => {
                                // Phase one locking is done, so we reply with an ack
                                &stream.write_all(bincode::serialize(&PhaseOneAck).unwrap().as_slice());

                                //Phase two starts, we listen for a commit or abort message
                                match read_request_message_from_stream(&stream) {
                                    Ok(msg) => {
                                        match msg {
                                            Commit => {
                                                match hashtable.insert(p.key, p.val) {
                                                    Ok(ret) => {
                                                        let ret_clone = ret.clone();
                                                        // We don't need to tell the client that the put actually happened, so we just update metrics and move on
                                                        if ret_clone { metrics.put_insert.fetch_add(1, Relaxed); } else { metrics.put_update.fetch_add(1, Relaxed); }
                                                    }
                                                    Err(_) => { panic!("Lock acquired, but failed to get lock in hashtable!"); }
                                                }
                                            }
                                            Abort => { metrics.failed_request.fetch_add(1, Relaxed); }
                                            _ => { panic!("Expected Commit or Abort request"); }
                                        }
                                    }
                                    Err(e) => { panic!("Error reading message from stream: {}", e.to_string()); }
                                }
                            }
                            Err(_) => {
                                &stream.write_all(bincode::serialize(&RequestFailed).unwrap().as_slice());
                                metrics.failed_request.fetch_add(1, Relaxed);
                            }
                        }
                    }
                    MultiPut(puts) => {
                        // We need to lock each bucket that involves one of the puts
                        match multi_put_helper(&mut stream, hashtable.clone(), lock_table.clone(), &hashtable.num_buckets,
                                               puts, 0, HashSet::new()) {
                            Commit => { metrics.multi_put.fetch_add(1, Relaxed); }
                            Abort => { metrics.failed_request.fetch_add(1, Relaxed); }
                            RequestFailed => {
                                &stream.write_all(bincode::serialize(&RequestFailed).unwrap().as_slice());
                                metrics.failed_request.fetch_add(1, Relaxed);
                            }
                            _ => { panic!("multi_put_helper() function returned something unexpected!"); }
                        }
                    }
                    _ => { panic!("Expected Get, Put or MultiPut request"); }
                }
                // These metrics are measured for every operation, successful or not
                metrics.total_time_operations.fetch_add(start_operation.elapsed().as_micros(), Relaxed);
                metrics.num_operations.fetch_add(1, Relaxed);
            }
            Err(e) => { println!("Connection possibly closed: {}", e); return; }
        }
    }
}

pub fn accept_client(port: &u64, hashtable: &mut Arc<Hashtable<String>>, _pool: &ThreadPool, metrics: Arc<Metrics>) {
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port.to_string()).unwrap();
    let lock_table : Arc<Locktable> = Arc::new(Locktable::new(hashtable.num_buckets));

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