use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;
use std::fs::File;

#[derive(Serialize, Deserialize, Clone)]
pub struct Properties {
    pub node_ips: Vec<Ipv4Addr>,
    pub server_port: u64,
    pub server_client_check_port: u64,
    pub dht_num_threads: usize,
    pub key_range: Vec<usize>,
    pub num_requests: u64,
    pub num_client_threads: usize,
    pub replication_degree: usize,
    pub multi_put_num: usize
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
        Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
        Ok(file) => file,
    };
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    println!("{}", data);
    serde_json::from_str(&data).unwrap()
}