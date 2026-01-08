/* ====================================================================================================
  PARALLEL PRIME NUMBER FINDER
====================================================================================================

OVERVIEW:
  A multi-threaded program that finds all prime numbers up to 100 million using atomic operations
  for lock-free synchronization. Demonstrates efficient parallel computation without traditional
  mutex locking mechanisms.

PROGRAM FLOW:
  1. Initialize shared atomic counters (work counter and result counter)
  2. Spawn 10 worker threads that compete for work
  3. Each thread atomically fetches the next number to check
  4. Thread tests if number is prime using optimized algorithm
  5. Thread accumulates local count, then adds to global total when done
  6. All threads complete, main thread aggregates and displays results

KEY STRUCTURES:
  • AtomicUsize counter: Shared work distributor (which number to check next)
  • AtomicUsize total_primes: Shared result accumulator
  • Arc: Enables safe shared ownership of atomics across threads

CORE FUNCTIONS:
  • is_prime(n): Optimized primality test using trial division up to sqrt(n)
    - Returns early for small numbers and even numbers
    - Checks divisibility by 6k±1 pattern for efficiency
  • main(): Orchestrates parallel prime search with work-stealing pattern

CONCURRENCY MECHANISM:
  • Atomic Operations: Lock-free thread-safe operations on shared counters
  • fetch_add(): Atomically reads current value, increments it, returns old value
  • Ordering::SeqCst: Strongest memory ordering (sequential consistency)
  • Ordering::Relaxed: Weaker ordering sufficient for simple accumulation

WORK DISTRIBUTION PATTERN:
  Threads use "work stealing" - each atomically claims the next available number to check.
  No explicit task queue needed; the atomic counter serves as an implicit work distributor.
  When counter exceeds limit, threads naturally exit their loops.

PERFORMANCE OPTIMIZATION:
  • Local accumulation reduces atomic contention (each thread updates global total once)
  • Lock-free atomics avoid mutex overhead
  • Prime algorithm optimized with early returns and 6k±1 pattern

==================================================================================================== */

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn is_prime(n: usize) -> bool {
    if n <= 1 { return false; }
    if n <= 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

fn main() {
    let limit = 100_000_000;
    let num_threads = 10;
    
    let counter = Arc::new(AtomicUsize::new(2));
    let total_primes = Arc::new(AtomicUsize::new(0));
    
    let mut handles = vec![];
    let start_total = Instant::now();

    for t in 0..num_threads {
        let counter_ref = Arc::clone(&counter);
        let total_ref = Arc::clone(&total_primes);

        let handle = thread::spawn(move || {
            let thread_start = Instant::now();
            let mut local_count = 0;

            loop {
                let num = counter_ref.fetch_add(1, Ordering::SeqCst);
                
                if num > limit { break; } 

                if is_prime(num) {
                    local_count += 1;
                }
            }

            total_ref.fetch_add(local_count, Ordering::Relaxed);
            
            println!("Thread {:2}: Finished in {:?}.", t, thread_start.elapsed());
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("--------------------------------------");
    println!("Total Primes Found: {}", total_primes.load(Ordering::SeqCst));
    println!("Total Execution Time: {:?}", start_total.elapsed());
}