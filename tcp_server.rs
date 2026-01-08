/* ====================================================================================================
  MULTI-THREADED WEB SERVER WITH THREAD POOL
====================================================================================================

OVERVIEW:
  A TCP web server that uses a fixed-size thread pool to handle concurrent
  HTTP requests efficiently. Demonstrates the thread pool pattern for limiting resource usage
  while maintaining high concurrency, along with graceful shutdown mechanisms.

PROGRAM FLOW:
  1. Server binds to port 7878 and creates a thread pool with 4 worker threads
  2. Main thread listens for incoming TCP connections in a loop
  3. For each connection, server dispatches handling task to thread pool
  4. Available worker thread picks up the task and processes the HTTP request
  5. Worker simulates slow/fast responses based on request path
  6. On shutdown, pool gracefully terminates all workers

ARCHITECTURE:
  • ThreadPool: Manages a fixed number of worker threads and distributes jobs
  • Worker: Individual thread that continuously waits for and executes jobs
  • Job: Type alias for boxed closures that can be sent across threads
  • mpsc::channel: Message passing queue for sending jobs to workers

KEY STRUCTURES:
  • ThreadPool:
    - workers: Vec of Worker threads
    - sender: Channel sender for dispatching jobs (Option for graceful drop)
  
  • Worker:
    - id: Unique identifier for logging
    - thread: JoinHandle wrapped in Option (allows taking ownership during shutdown)

CORE FUNCTIONS:
  • ThreadPool::new(size): Creates pool with specified number of workers
    - Sets up mpsc channel for job distribution
    - Wraps receiver in Arc<Mutex> so all workers can share it
    - Spawns worker threads that compete for jobs

  • ThreadPool::execute(f): Submits a closure to be executed by a worker
    - Boxes the closure to make it a Job type
    - Sends it through the channel to waiting workers

  • Worker::new(id, receiver): Spawns a thread that loops indefinitely
    - Locks receiver, waits for job, executes it
    - Exits loop when sender is dropped (graceful shutdown)

  • handle_connection(stream): Processes individual HTTP request
    - Reads request from TCP stream
    - Simulates slow response (5 sec) for /sleep endpoint
    - Returns immediate response for all other paths

CONCURRENCY PATTERNS:
  • Thread Pool Pattern: Fixed workers prevent thread explosion under load
  • Message Passing: Jobs sent via mpsc (multi-producer, single-consumer) channel
  • Shared Receiver: Arc<Mutex<Receiver>> allows workers to compete for jobs
  • Graceful Shutdown: Drop trait implementation ensures clean worker termination

GRACEFUL SHUTDOWN MECHANISM:
  When ThreadPool is dropped:
  1. sender is explicitly dropped via take(), closing the channel
  2. All workers' recv() calls return Err, breaking their loops
  3. Main thread joins each worker thread, waiting for completion
  4. Ensures no jobs are abandoned and all resources are cleaned up

ENDPOINT BEHAVIOR:
  • GET /sleep: Returns after 5-second delay (simulates slow operation)
  • Any other path: Returns immediately
  • All responses are HTTP 200 OK with plain text content

WHY THREAD POOL OVER SPAWN-PER-REQUEST:
  • Limits resource usage (memory, CPU context switching)
  • Prevents DOS via thread exhaustion
  • Reuses threads, avoiding spawn/teardown overhead
  • Provides backpressure when all workers are busy

==================================================================================================== */

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    println!("Server running on 127.0.0.1:7878 with 4 worker threads");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection established!");

                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(_) => {
            println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

            let get_sleep = b"GET /sleep HTTP/1.1\r\n";

            let (status_line, content) = if buffer.starts_with(get_sleep) {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "Slow Response\n")
            } else {
                ("HTTP/1.1 200 OK", "Fast Response\n")
            };

            let response = format!(
                "{}\r\nContent-Length: {}\r\n\r\n{}",
                status_line,
                content.len(),
                content
            );

            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Failed to write response: {}", e);
                return;
            }

            if let Err(e) = stream.flush() {
                eprintln!("Failed to flush stream: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}