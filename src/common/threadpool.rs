use std::thread;
use std::sync::{mpsc, Arc, Mutex};

// Made by following tutorial at: https://doc.rust-lang.org/book/ch20-02-multithreaded.html
#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // creates a channel to send and receive jobs to/from
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver)); // Arc::Mutex because only one worker should get a job at a time
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            println!("Worker {} ready!", id);
            // Each worker repeatedly tries to receive a job, executing the job if it gets one
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                //println!("Worker {} got a job; executing.", id);
                job();
            }
        });

        Worker {
            id,
            thread,
        }
    }
}