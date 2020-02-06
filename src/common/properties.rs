use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;
use std::fs::File;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Properties {
    pub NUM_NODES: u64,
    pub NODE_IPS: Vec<Ipv4Addr>,
    pub DHT_NUM_THREADS: u64,
    pub DHT_NUM_BUCKETS: u64,
}

/**
* reads from the properties.json file to load the properties for the DHT
* returns Properties
**/
pub fn getProperties() -> Properties{
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