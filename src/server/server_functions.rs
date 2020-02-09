use std::net::{TcpStream, TcpListener};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream};
use std::sync::Arc;
use mylib::common::threadpool::ThreadPool;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

pub fn handle_client(stream: TcpStream, hashtable: Arc<Hashtable<String>>) {
    match read_request_message_from_stream(&stream) {
        Ok(msg) => {
            let response: DHTMessage;
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
                _ => { panic!("Expected Get or Put request"); }
            }
            serde_json::to_writer(&stream, &response).unwrap();
        }
        Err(e) => { println!("Error: {}", e); }
    }
}

pub fn accept_client(hashtable: &mut Arc<Hashtable<String>>, pool: &ThreadPool) {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        //println!("Connection accepted! Handling...");
        match stream {
            Ok(stream) => {
                let hashtable_clone = hashtable.clone();
                pool.execute(|| { handle_client(stream, hashtable_clone) });
                //thread::spawn(move || { handle_client(stream, hashtable_clone, msg) });
                //println!("Done handling, listening again...")
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
}