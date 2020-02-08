use std::collections::HashMap;



pub struct Hashtable<T> {
    num_buckets: usize,
    hashtable: HashMap<u64, T>,
}

impl Hashtable<String> {
    pub fn new(num_buckets: usize) -> Hashtable<String> {
        Hashtable {
            num_buckets,
            hashtable: HashMap::new(),
        }
    }

    pub fn get(&self, key: &u64) -> Option<String> {
        match self.hashtable.get(key) {
            Some(msg) => { Some(msg.clone()) }
            None => { None }
        }

    }

    pub fn insert(&mut self, key: u64, val: String) -> bool {
        match self.hashtable.get(&key) {
            Some(_) => {
                false
            }
            None => {
                self.hashtable.insert(key, val);
                true
            }
        }
    }
}