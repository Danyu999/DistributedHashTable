struct Bucket<T> {
    contents: Vec<(String, T)>,
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

    pub fn get(&self, key: &String, bucket_index: usize) -> Option<String> {
        let bucket = &self.buckets[bucket_index];
        for i in 0..bucket.contents.len() {
            if bucket.contents[i].0 == *key {
                return Some(bucket.contents[i].1.clone());
            }
        }
        return None;
    }

    // returns false if value already in hashtable and was updated, true if inserted
    pub fn insert(&mut self, key: String, val: String, bucket_index: usize) -> bool {
        let mut bucket = &mut self.buckets[bucket_index];
                // Check if the key is already in the bucket, if so, then the put fails and we return false
                for i in 0..bucket.contents.len() {
                    if bucket.contents[i].0 == key {
                        bucket.contents[i].1 = val;
                        return false;
                    }
                }

                // If we reach here, that means the key doesn't exist yet, so we add it
                bucket.contents.push((key, val));
                return true;
    }
}