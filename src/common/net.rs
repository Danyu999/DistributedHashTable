use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::io::{Read, Write};
use std::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum MessageType<T> {
    Get(u64), //(key)
    Put(u64, u64, T) //(key, sizeOfContent, content)
}

//from https://docs.serde.rs/serde_json/de/fn.from_reader.html
fn readMessageFromStream(stream: &TcpStream) -> Result<MessageType<T>, Box<dyn Error>> {
    let mut de = serde_json::Deserializer::from_reader(stream);
    let u = MessageType::deserialize(&mut de)?;

    Ok(u)
}