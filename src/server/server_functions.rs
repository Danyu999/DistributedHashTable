use std::net::{TcpStream, TcpListener};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream};
use std::sync::Arc;
use mylib::common::threadpool::ThreadPool;
use std::thread;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

pub fn handle_client(stream: TcpStream, hashtable: Arc<Hashtable<String>>) {
    loop {
        match read_request_message_from_stream(&stream) {
            Ok(msg) => {
                //println!("Handling a message");
                let mut response = DHTMessage::RequestFailed;
                match msg {
                    DHTMessage::Get(key) => {
                        match hashtable.get(&key) {
                            Ok(val) => { response = DHTMessage::GetResponse(val); }
                            Err(_) => { response = DHTMessage::RequestFailed; }
                        }
                    }
                    DHTMessage::Put(key, val) => {
                        match hashtable.insert(key, val) {
                            Ok(success) => { response = DHTMessage::PutResponse(success); }
                            Err(_) => { response = DHTMessage::RequestFailed; }
                        }
                    }
                    _ => { println!("Expected Get or Put request"); }
                }
                serde_json::to_writer(&stream, &response).unwrap();
            }
            Err(e) => { println!("Error: {}", e); return; }
        }
    }
}

pub fn accept_client(port: &u64, hashtable: &mut Arc<Hashtable<String>>, _pool: &ThreadPool) {
    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port.to_string()).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port {}", &port);
    for stream in listener.incoming() {
        //println!("Connection accepted! Handling...");
        match stream {
            Ok(stream) => {
                let hashtable_clone = hashtable.clone();
                // pool.execute(|| { handle_client(stream, hashtable_clone) });
                thread::spawn(move || { handle_client(stream, hashtable_clone) });
                //println!("Done handling, listening again...")
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
}