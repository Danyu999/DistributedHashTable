pub struct Metrics {
    pub key_range_size: u64,
    pub successful_put: u64,
    pub unsuccessful_put: u64,
    pub some_get: u64,
    pub none_get: u64,
    pub failed_request: u64,
    pub total_time: u128,
    pub num_operations: u64,
    pub time_one_operation: Vec<u128>,
} //TODO: consider measuring how throughput changes throughout execution (does it change due there being a higher likelihood of contention as the program continues?)

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            key_range_size: 0,
            successful_put: 0,
            unsuccessful_put: 0,
            some_get: 0,
            none_get: 0,
            failed_request: 0,
            total_time: 0,
            num_operations: 0,
            time_one_operation: Vec::new(),
        }
    }
}