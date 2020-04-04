use std::sync::Mutex;
use crate::common::my_hash;

struct Bucket<T> {
    contents: Vec<(u64, T)>,
}

impl<T> Bucket<T> {
    fn new() -> Bucket<T> {
        Bucket {
            contents: Vec::new(),
        }
    }
}

pub struct Hashtable<T> {
    pub num_buckets: usize,
    buckets: Vec<Bucket<T>>,
}

impl Hashtable<String> {
    pub fn new(num_buckets: usize) -> Hashtable<String> {
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket::new());
        }

        Hashtable {
            num_buckets,
            buckets,
        }
    }

    pub fn get(&self, key: &u64, bucket_index: &usize) -> Option<String> {
        let bucket = &self.buckets[bucket_index];
        for i in 0..bucket.contents.len() {
            if bucket.contents[i].0 == *key {
                return Some(bucket.contents[i].1.clone());
            }
        }
        return None;
    }

    pub fn insert(&self, key: u64, val: String) -> Result<bool, &'static str> {
        let bucket_index: usize = my_hash(key) as usize % self.num_buckets;
        let bucket = &self.buckets[bucket_index];
        return match bucket.try_lock() {
            Ok(mut mutex_bucket) => {
                // Check if the key is already in the bucket, if so, then the put fails and we return false
                for i in 0..mutex_bucket.contents.len() {
                    if mutex_bucket.contents[i].0 == key {
                        return Ok(false);
                    }
                }

                // If we reach here, that means the key doesn't exist yet, so we add it
                mutex_bucket.contents.push((key, val));
                Ok(true)
            }
            Err(_) => { Err("Lock taken, request denied") }
        }
    }
}