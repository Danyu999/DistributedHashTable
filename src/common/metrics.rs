use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicU64;
use atomic::Atomic;
use std::fs::OpenOptions;
use std::io::Write;

pub struct Metrics {
    pub put_insert: AtomicU64,
    pub put_update: AtomicU64,
    pub some_get: AtomicU64,
    pub none_get: AtomicU64,
    pub multi_put: AtomicU64,
    pub failed_request: AtomicU64,
    pub num_operations: AtomicU64,
    pub total_time_operations: Atomic<u128>,
    total_time_elapsed: Atomic<u128>,
} //TODO: consider measuring how throughput changes throughout execution (does it change due there being a higher likelihood of contention as the program continues?)

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            put_insert: AtomicU64::new(0),
            put_update: AtomicU64::new(0),
            some_get: AtomicU64::new(0),
            none_get: AtomicU64::new(0),
            multi_put: AtomicU64::new(0),
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
        println!("Total time elapsed: {} seconds", metrics.total_time_elapsed.load(Relaxed) as f64 / 1000000 as f64);
        println!("Put inserts: {}", metrics.put_insert.load(Relaxed));
        println!("Put updates: {}", metrics.put_update.load(Relaxed));
        println!("Some gets: {}", metrics.some_get.load(Relaxed));
        println!("None gets: {}", metrics.none_get.load(Relaxed));
        println!("Multi puts: {}", metrics.multi_put.load(Relaxed));
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
            println!("Average time for system to accomplish one operation: N/A");
        }
        else {
            println!("Average time for system to accomplish one operation: {} microseconds",
                 metrics.total_time_operations.load(Relaxed) / metrics.num_operations.load(Relaxed) as u128);
        }
        println!();
    }
}

fn print_metrics_file(metrics: &Metrics) -> String {
    let mut res = String::new();
    // res.push_str("Total number of operations: ");
    // res.push_str(metrics.num_operations.load(Relaxed).to_string().as_ref());
    // res.push_str("\n");
    // res.push_str("Total time elapsed: ");
    res.push_str((metrics.total_time_elapsed.load(Relaxed) as f64 / 1000000 as f64).to_string().as_ref());
    res.push_str("\n");
    // res += "Put inserts: " + metrics.put_insert.load(Relaxed) + "\n";
    // res += "Put updates: " + metrics.put_update.load(Relaxed) + "\n";
    // res += "Some gets: " + metrics.some_get.load(Relaxed) + "\n";
    // res += "None gets: " + metrics.none_get.load(Relaxed) + "\n";
    // res += "Multi puts: " + metrics.multi_put.load(Relaxed) + "\n";
    // res.push_str("Failed requests: ");
    // res.push_str(metrics.failed_request.load(Relaxed).to_string().as_ref());
    // res.push_str("\n");
    // res.push_str("Total send_requests time: ");
    // res.push_str((metrics.total_time_operations.load(Relaxed) as f64 / 1000000 as f64).to_string().as_ref());
    // res.push_str("\n");
    if metrics.total_time_operations.load(Relaxed) == 0 {
        res += "N/A\n";
    }
    else {
        // res.push_str("Average number of operations per second: ");
        res.push_str((metrics.num_operations.load(Relaxed) as f64 / (metrics.total_time_operations.load(Relaxed) as f64 / 1000000 as f64)).to_string().as_ref());
        res.push_str("\n");
    }
    if metrics.num_operations.load(Relaxed) == 0 {
        res += "N/A\n";
    }
    else {
        // res.push_str("Average time for system to accomplish one operation: ");
        res.push_str((metrics.total_time_operations.load(Relaxed) / metrics.num_operations.load(Relaxed) as u128).to_string().as_ref());
        res.push_str("\n\n");
    }
    return res;
}

// Wakes up every now and then to gather metrics
pub fn gather_metrics(mut metrics_list: Vec<Metrics>, metrics: Arc<Metrics>, stop_server: Arc<AtomicBool>, stop_server_followup_clone: Arc<AtomicBool>) {
    let start = Instant::now();
    loop {
        sleep(Duration::new(20,0));
        println!("Gathering metrics...");
        let metrics_clone = Metrics::new();
        metrics_clone.total_time_elapsed.store(start.elapsed().as_micros(), Relaxed);
        metrics_clone.put_insert.store(metrics.put_insert.load(Relaxed), Relaxed);
        metrics_clone.put_update.store(metrics.put_update.load(Relaxed), Relaxed);
        metrics_clone.some_get.store(metrics.some_get.load(Relaxed), Relaxed);
        metrics_clone.none_get.store(metrics.none_get.load(Relaxed), Relaxed);
        metrics_clone.multi_put.store(metrics.multi_put.load(Relaxed), Relaxed);
        metrics_clone.failed_request.store(metrics.failed_request.load(Relaxed), Relaxed);
        metrics_clone.num_operations.store(metrics.num_operations.load(Relaxed), Relaxed);
        metrics_clone.total_time_operations.store(metrics.total_time_operations.load(Relaxed), Relaxed);

        // Append to a file
        let mut file = OpenOptions::new().append(true).open("metrics.txt").unwrap();
        file.write_all(print_metrics_file(&metrics_clone).as_bytes()).unwrap();

        metrics_list.push(metrics_clone);



        // If a stop_server signal has been sent, we print the metrics and crash the server
        if stop_server.load(Relaxed) {
            print_metrics(metrics_list);
            stop_server_followup_clone.store(true, Relaxed);
            return;
        }
    }

}