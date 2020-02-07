use std::collections::HashMap;



pub struct Hashtable<T> {
    num_buckets: u64,
    hashtable: HashMap<u64, T>,
}

impl<T> Hashtable<T> {
    pub fn new() -> Hashtable<T> {
        Hashtable {
            num_buckets: 10,
            hashtable: HashMap::new(),
        }
    }
}