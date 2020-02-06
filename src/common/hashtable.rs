use std::collections::HashMap;



pub struct Hashtable {
    num_buckets: u64,
    hashtable: HashMap<u64, String>,
}

impl Hashtable {
    pub fn new() -> Hashtable {
        Hashtable {
            num_buckets: 10,
            hashtable: HashMap::new(),
        }
    }
}