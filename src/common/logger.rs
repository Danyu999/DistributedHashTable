use std::fs::{File, OpenOptions};
use crate::common::net::DHTMessage;
use crate::common::net::DHTMessage::{Put, MultiPut};
use std::io::Write;

pub struct Log {
    file_descriptor: File
}

impl Log {
    pub fn client_new() -> Log {
        Log {
            file_descriptor: OpenOptions::new().append(true).open("log/client.log").unwrap()
        }
    }

    pub fn server_new() -> Log {
        Log {
            file_descriptor: OpenOptions::new().append(true).open("log/server.log").unwrap()
        }
    }

    // Logs the message. If it is an initial request_log message, it also logs the contents of the request.
    // Otherwise, it just logs the request id.
    fn log(&mut self, initial: bool, mut log: String, msg: &DHTMessage) {
        match msg {
            // Note: Get does not need logging since it doesn't use 2PC
            Put(p, id) => {
                log.push_str("PUT");
                log.push_str(id.to_string().as_ref());
                if initial {
                    log.push_str("\n");
                    log.push_str(p.key.as_ref());
                    log.push_str("\n");
                    log.push_str(p.val.as_ref());
                }
            }
            MultiPut(puts, id) => {
                log.push_str("MUL");
                log.push_str(id.to_string().as_ref());
                if initial {
                    log.push_str("\n");
                    log.push_str(puts.len().to_string().as_ref());
                    for p in puts {
                        log.push_str("\n");
                        log.push_str(p.key.as_ref());
                        log.push_str("\n");
                        log.push_str(p.val.as_ref());
                    }
                }
            }
            _ => { panic!("Expected Put or MulitPut for log request!"); }
        }
        log.push_str("\n");
        self.file_descriptor.write_all(log.as_bytes()).unwrap();
    }

    pub fn request_log(&mut self, msg: &DHTMessage) {
        self.log( true,String::from(""), msg);
    }

    pub fn commit_log(&mut self, msg: &DHTMessage) {
        self.log(false, String::from("CMT"), msg);
    }

    pub fn abort_log(&mut self, msg: &DHTMessage) {
        self.log(false, String::from("ABT"), msg);
    }
}
