use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub mod hashtable;
pub mod locktable;
pub mod net;
pub mod properties;
pub mod metrics;
pub mod threadpool;

// Hash function to hash the keys. Each DefaultHasher created with new is guaranteed to be the same as others
pub fn my_hash<T>(obj: T) -> u64 where T: Hash, {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}