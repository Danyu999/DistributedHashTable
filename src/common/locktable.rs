use std::sync::Mutex;

pub struct Locktable {
    pub locks: Vec<Mutex<bool>>
}

impl Locktable {
    pub fn new(num_buckets: usize) -> Locktable {
        let mut locks = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            locks.push(Mutex::new(false));
        }

        Locktable {
            locks,
        }
    }
}