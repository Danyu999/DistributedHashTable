use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;
use std::fs::File;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Properties {
    pub node_ips: Vec<Ipv4Addr>,
    pub server_port: u64,
    pub dht_num_threads: usize,
    pub dht_num_buckets: usize,
    pub key_range: Vec<u64>,
    pub num_requests: u64
}

/**
* reads from the properties.json file to load the properties for the DHT
* returns Properties
**/
pub fn get_properties() -> Properties{
    let path = Path::new("src/properties.json");
    let display = path.display();
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    println!("{}", data);
    serde_json::from_str(&data).unwrap()
}