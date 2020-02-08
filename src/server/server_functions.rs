use std::net::{TcpStream, TcpListener};
use mylib::common::hashtable::Hashtable;
use mylib::common::net::{DHTMessage, read_request_message_from_stream};
use std::sync::{Arc, Mutex};
use std::thread;

/**
* server_functions handles parsing the input from a client, and calling the respective server command
**/

pub fn handle_client(stream: TcpStream, hashtable: Arc<Mutex<Hashtable<String>>>, msg: DHTMessage) {
    let response: DHTMessage;
    match hashtable.try_lock() {
        Ok(mut mutex_ht) => {
            match msg {
                DHTMessage::Get(key) => { response = DHTMessage::GetResponse(mutex_ht.get(&key)); }
                DHTMessage::Put(key, val) => { response = DHTMessage::PutResponse(mutex_ht.insert(key, val)); }
                _ => { panic!("Expected Get or Put request"); }
            }
        }
        Err(_) => { response = DHTMessage::RequestFailed; }
    }
    serde_json::to_writer(&stream, &response).unwrap();
}

pub fn accept_client(hashtable: &mut Arc<Mutex<Hashtable<String>>>) {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        //println!("Connection accepted! Handling...");
        match stream {
            Ok(stream) => {
                match read_request_message_from_stream(&stream) {
                    Ok(msg) => {
                        let hashtable_clone = hashtable.clone();
                        thread::spawn(move || { handle_client(stream, hashtable_clone, msg) });
                        //println!("Done handling, listening again...")
                    }
                    Err(e) => { println!("Error: {}", e); }
                }
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
    // close the socket server
    drop(listener);
}