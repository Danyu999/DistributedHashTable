use std::net::{TcpStream, TcpListener};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream, PutRequest, write_dht_message_to_stream};
use std::sync::{Arc, Mutex};
use mylib::common::threadpool::ThreadPool;
use mylib::common::metrics::Metrics;
use std::time::Instant;
use std::sync::atomic::Ordering::Relaxed;
use mylib::common::my_hash;
use mylib::common::net::DHTMessage::{PhaseOneAck, GetResponse, Commit, Abort, RequestFailed, Get, Put, MultiPut};
use mylib::common::locktable::Locktable;
use std::collections::HashSet;
use mylib::common::logger::Log;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

// Possible implementation idea for MultiPut where we recursively acquire the locks; This method would allow MultiPut to have
// returns Commit if successful, Abort if aborted, and RequestFailed if failed (3 different behaviors for metrics)
// This method does all the stream writing and reading
fn multi_put_helper(mut stream: &mut TcpStream, hashtable: Arc<Hashtable<String>>, lock_table: Arc<Locktable>, num_buckets: &usize,
                    puts: Vec<PutRequest>, lock_progress: usize, mut prev_buckets: HashSet<usize>, logger: &Arc<Mutex<Log>>, msg_clone: &DHTMessage) -> DHTMessage {
    // We recursively try to acquire the locks for each put operation, then continues the 2PL while holding all locks
    return if lock_progress == puts.len() {
        //Log the message (We use curly braces here so that the lock is released right after this log)
        { logger.lock().unwrap().request_log(&msg_clone); }

        // We continue the operation since we have all the locks
        // Phase one locking is done, so we reply with an ack
        write_dht_message_to_stream(&mut stream, &PhaseOneAck);
        match read_request_message_from_stream(&mut stream) {
            Ok(msg) => {
                match msg {
                    Commit => {
                        //Log the message (We use curly braces here so that the lock is released right after this log)
                        { logger.lock().unwrap().commit_log(&msg_clone); }

                        for p in puts {
                            match hashtable.insert(p.key, p.val) {
                                Ok(_) => {} //As long as it succeeds, we don't care whether the put inserted or updated
                                Err(_) => { panic!("Lock acquired, but failed to get lock in hashtable!"); }
                            }
                        }
                        // let t4 = Instant::now();
                        write_dht_message_to_stream(&mut stream, &Commit);
                        // println!("server write commit: {}", t4.elapsed().as_micros());
                        Commit
                    }
                    Abort => { Abort }
                    m => { panic!("Expected Commit or Abort request, got: {}", m); }
                }
            }
            Err(e) => { panic!("Error reading message from stream: {}", e.to_string()); }
        }
    } else {
        // We continue acquiring locks
        let bucket_index: usize = my_hash(puts[lock_progress].key.as_str()) as usize % *num_buckets;
        if prev_buckets.contains(&bucket_index) {
            multi_put_helper(stream, hashtable.clone(), lock_table.clone(), &num_buckets, puts,
                             lock_progress + 1, prev_buckets, logger, msg_clone)
        } else {
            // Lock not acquired yet, so we try to acquire
            match lock_table.locks[bucket_index].try_lock() {
                Ok(_) => {
                    prev_buckets.insert(bucket_index);
                    multi_put_helper(stream, hashtable.clone(), lock_table.clone(), &num_buckets, puts,
                                     lock_progress + 1, prev_buckets, logger, msg_clone)
                }
                Err(_) => {
                    //Log the message (We use curly braces here so that the lock is released right after this log)
                    { logger.lock().unwrap().abort_log(&msg_clone); }
                    RequestFailed
                }
            }
        }
    }
}

pub fn handle_client(mut stream: TcpStream, hashtable: Arc<Hashtable<String>>, lock_table: Arc<Locktable>, metrics: Arc<Metrics>, logger: Arc<Mutex<Log>>) {
    let mut start_operation: Instant;
    let mut failed_op_time = 0;
    let mut success = true;
    loop {
        let read_op_time = Instant::now();
        let message  = match bincode::deserialize_from(&stream) {
            Ok(msg) => { Ok(msg) }
            Err(e) => { Err(e) }
        };
        // If the previous op failed, that means this op will be a retry, so we want to count the time it takes to read
        if !success { failed_op_time += read_op_time.elapsed().as_micros(); }
        success = true;

        match message {
            Ok(msg) => {
                start_operation = Instant::now();
                match msg {
                    Get(key, _) => {
                        // println!("Read get: {}", t1.elapsed().as_micros());
                        // Get request, so we don't do 2PL
                        let bucket_index: usize = my_hash(key.as_str()) as usize % hashtable.num_buckets;
                        match lock_table.locks[bucket_index].try_lock() {
                            // The lock is not taken, so we lock
                            Ok(_) => {
                                match hashtable.get(&key) {
                                    Ok(val) => {
                                        let val_clone = val.clone();
                                        // let t4 = Instant::now();
                                        write_dht_message_to_stream(&mut stream, &GetResponse(val));
                                        // println!("server write get: {}", t4.elapsed().as_micros());
                                        if val_clone == None { metrics.some_get.fetch_add(1, Relaxed); }
                                        else { metrics.none_get.fetch_add(1, Relaxed); }
                                    }
                                    Err(_) => { panic!("Lock acquired for get, but failed to get lock in hashtable!"); }
                                }
                            }
                            Err(_) => {
                                write_dht_message_to_stream(&mut stream, &RequestFailed);
                                metrics.failed_request.fetch_add(1, Relaxed);
                                success = false;
                            }
                        }
                    }
                    Put(p, id) => {
                        // println!("server read put: {}", t1.elapsed().as_micros());
                        let bucket_index: usize = my_hash(p.key.as_str()) as usize % hashtable.num_buckets;
                        let msg_clone = Put(p.clone(), id.clone());
                        match lock_table.locks[bucket_index].try_lock() {
                            // The lock is not taken, so we lock
                            Ok(_) => {
                                //Log the message (We use curly braces here so that the lock is released right after this log)
                                { logger.lock().unwrap().request_log(&msg_clone); }

                                // Phase one locking is done, so we reply with an ack
                                // let t2 = Instant::now();
                                write_dht_message_to_stream(&mut stream, &PhaseOneAck);
                                // println!("server write get: {}", t2.elapsed().as_micros());

                                //Phase two starts, we listen for a commit or abort message
                                match read_request_message_from_stream(&mut stream) {
                                    Ok(msg) => {
                                        // println!("server read commit/abort get: {}", t3.elapsed().as_micros());
                                        match msg {
                                            Commit => {
                                                //Log the message (We use curly braces here so that the lock is released right after this log)
                                                { logger.lock().unwrap().commit_log(&msg_clone); }

                                                match hashtable.insert(p.key, p.val) {
                                                    Ok(ret) => {
                                                        let ret_clone = ret.clone();
                                                        // let t4 = Instant::now();
                                                        write_dht_message_to_stream(&mut stream, &Commit);
                                                        // println!("server write commit: {}", t4.elapsed().as_micros());
                                                        if ret_clone { metrics.put_insert.fetch_add(1, Relaxed); } else { metrics.put_update.fetch_add(1, Relaxed); }
                                                    }
                                                    Err(_) => { panic!("Lock acquired, but failed to get lock in hashtable!"); }
                                                }
                                            }
                                            Abort => {
                                                metrics.failed_request.fetch_add(1, Relaxed);
                                            }
                                            m => { panic!("Expected Commit or Abort request, instead got: {}", m); }
                                        }
                                    }
                                    Err(e) => { panic!("Error reading message from stream: {}", e.to_string()); }
                                }
                            }
                            Err(_) => {
                                //Log the message (We use curly braces here so that the lock is released right after this log)
                                { logger.lock().unwrap().abort_log(&msg_clone); }

                                write_dht_message_to_stream(&mut stream, &RequestFailed);
                                metrics.failed_request.fetch_add(1, Relaxed);
                                success = false;
                            }
                        }
                    }
                    MultiPut(puts, id) => {
                        let msg_clone = MultiPut(puts.clone(), id.clone());
                        // println!("t1 multiput: {}", t1.elapsed().as_micros());
                        // We need to lock each bucket that involves one of the puts
                        match multi_put_helper(&mut stream, hashtable.clone(), lock_table.clone(), &hashtable.num_buckets,
                                               puts, 0, HashSet::new(), &logger, &msg_clone) {
                            Commit => { metrics.multi_put.fetch_add(1, Relaxed); }
                            Abort => { metrics.failed_request.fetch_add(1, Relaxed); }
                            RequestFailed => {
                                write_dht_message_to_stream(&mut stream, &RequestFailed);
                                metrics.failed_request.fetch_add(1, Relaxed);
                                success = false;
                            }
                            _ => { panic!("multi_put_helper() function returned something unexpected!"); }
                        }
                    }
                    m => { panic!("Expected Get, Put or MultiPut request, got: {} ", m); }
                }
                // These metrics are measured for every operation, successful or not. We accumulate failed attempts at the same operation
                if success {
                    metrics.total_time_operations.fetch_add(start_operation.elapsed().as_micros() + failed_op_time, Relaxed);
                    metrics.num_operations.fetch_add(1, Relaxed);
                    failed_op_time = 0;
                } else {
                    failed_op_time += start_operation.elapsed().as_micros();
                }
            }
            Err(_e) => { /*println!("Connection possibly closed server-side: {}", e);*/ return; }
        }
    }
}

pub fn accept_client(port: &u64, hashtable: &mut Arc<Hashtable<String>>, _pool: &ThreadPool, metrics: Arc<Metrics>, logger: Arc<Mutex<Log>>) {
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port.to_string()).unwrap();
    let lock_table : Arc<Locktable> = Arc::new(Locktable::new(hashtable.num_buckets));
    let mut thread_num = 0;

    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port {}", &port);
    for stream in listener.incoming() {
        //println!("Connection accepted! Handling...");
        match stream {
            Ok(stream) => {
                let hashtable_clone = hashtable.clone();
                let metrics_clone = metrics.clone();
                let lock_table_clone = lock_table.clone();
                let logger_clone = logger.clone();
                let mut name = String::from("server thread #");
                name.push_str(thread_num.to_string().as_ref());
                thread_num += 1;
                // pool.execute(|| { handle_client(stream, hashtable_clone) });
                std::thread::Builder::new().name(name)
                    .spawn(move || { handle_client(stream, hashtable_clone, lock_table_clone, metrics_clone, logger_clone) }).unwrap();
                //println!("Done handling, listening again...")
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
}