use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicU64;
use atomic::Atomic;

pub struct Metrics {
    pub successful_put: AtomicU64,
    pub unsuccessful_put: AtomicU64,
    pub some_get: AtomicU64,
    pub none_get: AtomicU64,
    pub failed_request: AtomicU64,
    pub num_operations: AtomicU64,
    pub total_time_operations: Atomic<u128>,
    total_time_elapsed: Atomic<u128>,
} //TODO: consider measuring how throughput changes throughout execution (does it change due there being a higher likelihood of contention as the program continues?)

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            successful_put: AtomicU64::new(0),
            unsuccessful_put: AtomicU64::new(0),
            some_get: AtomicU64::new(0),
            none_get: AtomicU64::new(0),
            failed_request: AtomicU64::new(0),
            num_operations: AtomicU64::new(0),
            total_time_operations: Atomic::new(0),
            total_time_elapsed: Atomic::new(0),
        }
    }
}

// Prints out the collected metrics
// TODO: write results to a file
pub fn print_metrics(metrics_list: Vec<Metrics>) {
    for metrics in metrics_list {
        println!("Total number of operations: {}", metrics.num_operations.load(Relaxed));
        println!("Successful puts: {}", metrics.successful_put.load(Relaxed));
        println!("Unsuccessful puts: {}", metrics.unsuccessful_put.load(Relaxed));
        println!("Some gets: {}", metrics.some_get.load(Relaxed));
        println!("None gets: {}", metrics.none_get.load(Relaxed));
        println!("Failed requests: {}", metrics.failed_request.load(Relaxed));
        println!("Total send_requests time: {} seconds",
                 metrics.total_time_operations.load(Relaxed) as f64 / 1000000 as f64);
        if metrics.total_time_operations.load(Relaxed) == 0 {
            println!("Average number of operations per second: N/A");
        }
        else {
            println!("Average number of operations per second: {}",
                 metrics.num_operations.load(Relaxed) as f64 / (metrics.total_time_operations.load(Relaxed) as f64 / 1000000 as f64));
        }
        if metrics.num_operations.load(Relaxed) == 0 {
            println!("Average time (microseconds) for system to accomplish one operation: N/A");
        }
        else {
            println!("Average time (microseconds) for system to accomplish one operation: {}",
                 metrics.total_time_operations.load(Relaxed) / metrics.num_operations.load(Relaxed) as u128);
        }
    }
}

// Wakes up every now and then to gather metrics
pub fn gather_metrics(mut metrics_list: Vec<Metrics>, metrics: Arc<Metrics>, stop_server: Arc<AtomicBool>, stop_server_followup_clone: Arc<AtomicBool>) {
    let start = Instant::now();
    loop {
        sleep(Duration::new(5,0));
        println!("Gathering metrics...");
        let metrics_clone = Metrics::new();
        metrics_clone.total_time_elapsed.store(start.elapsed().as_micros(), Relaxed);
        metrics_clone.successful_put.store(metrics.successful_put.load(Relaxed), Relaxed);
        metrics_clone.unsuccessful_put.store(metrics.unsuccessful_put.load(Relaxed), Relaxed);
        metrics_clone.some_get.store(metrics.some_get.load(Relaxed), Relaxed);
        metrics_clone.none_get.store(metrics.none_get.load(Relaxed), Relaxed);
        metrics_clone.failed_request.store(metrics.failed_request.load(Relaxed), Relaxed);
        metrics_clone.num_operations.store(metrics.num_operations.load(Relaxed), Relaxed);
        metrics_clone.total_time_operations.store(metrics.total_time_operations.load(Relaxed), Relaxed);
        metrics_list.push(metrics_clone);

        // If a stop_server signal has been sent, we print the metrics and crash the server
        if stop_server.load(Relaxed) {
            print_metrics(metrics_list);
            stop_server_followup_clone.store(true, Relaxed);
            return;
        }
    }

}