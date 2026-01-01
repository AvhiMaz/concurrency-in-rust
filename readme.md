# Rust Multithreading & Concurrency

A collection of example programs exploring multithreading and concurrency concepts in Rust.

## About

This repository contains practical examples demonstrating:

- Multithreading with `std::thread`
- Shared state with `Arc` and atomic types
- Thread synchronization
- Parallel computation
- Concurrency patterns in Rust

Each program is a self-contained example showcasing different aspects of concurrent programming.

## Running the Examples

Since these are standalone Rust files without Cargo, compile and run them directly:

```bash
# Compile
rustc filename.rs

# Run
./filename
```

### Example:

```bash
rustc prime.rs
./prime
```

On Windows:

```bash
rustc prime.rs
prime.exe
```

## Examples

- **prime.rs** - Parallel prime number counter using atomic operations and thread pools

More examples coming soon!

## Requirements

- Rust compiler (rustc)

Install Rust from [rustup.rs](https://rustup.rs/)

---

Feel free to explore, experiment, and learn!
