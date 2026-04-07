# Lesson 2: Shared State and Synchronization
## Tools
### 1. `Arc<T>` - Atomic Reference Counted
Prevents data from being dropped while it is still being used by other threads. Note: this is immutable. Purely for sharing data accross threads.
#### What is `Arc<T>`?
This allows multiple threads to own the same data. This is necessary because of Rust's ownership and lifetime rules. If you want multiple threads to access the same data and only one thread owns the data then when that thread drops the data there will be issues with other threads trying to access that memory. `Arc<T>` solves this problem by keeping track of how many references there are to the data and only dropping it when the last reference is dropped. This is not a mutex, so it does not provide mutability. It is purely for sharing data across threads.
#### How to use
> Creates a new `Arc<T>` instance for the data `5`.
```rust
let data = Arc::new(5);
```
> Creates a clone of the data which also has ownership of it.
```rust
let data2 = Arc::clone(&data);
```
### 2. `Mutex<T>` - Mutual Exclusion
As opposed to `Arc<T>`, mutex only allows one thread to access data at a time. This is useful when you have mutable data that needs to be shared across threads.
#### How to use
> Creates a new `Mutex<T>` instance for the data `0`
```rust
let data = Mutex::new(0);
```

> Create a lock on the data and access it. This will block other threads from accessing the data until the lock is released. Then when the lock goes out of scope it will automatically release the lock.
```rust
let mut guard = data.lock().unwrap(); // This will block other threads from accessing the data
*guard += 1; // Must dereference the guard to access the data inside the mutex
```

### Combining `Arc<T>` and `Mutex<T>`
When you want to share mutable data across threads, you can combine these to give all threads ownership (`Arc<T>`) and then allow one thread to mutate at a time (`Mutex<T>`).
```rust
let shared_data = Arc::new(Mutex::new(0));
```

## Test 1
### Objective
- Create a shared counter
- Spawn 4 threads
- Each thread
  - Loops 5 times
  - sleeps (this represents some work)
  - increments the shared counter
- Wait for all threads to finish
- Print final count

### Attempt 1
```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

fn main() {
    let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let mut handles: Vec<JoinHandle<()>> = vec![];
    for i in 0..4 {
        let thread_counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..5 {
                thread::sleep(Duration::from_secs(2));
                let mut mutable_counter = thread_counter.lock().unwrap(); // Important to lock **AFTER** sleep. Otherwise it will remain locked throughout sleep, killing parallelism
                *mutable_counter += 1;
                println!("Thread {} incremented counter", i);
            }
        }))
    }
    for handle in handles {
        handle.join().unwrap()
    }
    println!("{:?}", counter.lock().unwrap())
}
```
#### Output
```
Thread 0 incremented counter
Thread 3 incremented counter
Thread 2 incremented counter
...
Thread 3 incremented counter
Thread 1 incremented counter
Thread 2 incremented counter
20
```
## Test 2
Have a list which each thread pushes a result to, and then it is printed at the end.
- Loop 5 times
- Simulate work (sleep)
- Create a result string like: `"Thread 2 processed item 3"`
- Push it into the shared vector
After all threads finish:
- Print ALL results
- Print total count
### Attempt 1
```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

fn main() {
    let output: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles: Vec<JoinHandle<()>> = vec![];
    for i in 0..4 {
        let output_clone = Arc::clone(&output);
        handles.push(thread::spawn(move || {
            for _ in 0..5 {
                thread::sleep(Duration::from_secs(2));
                let mut mutable_output = output_clone.lock().unwrap();
                mutable_output.push(format!("Thread {} incremented counter", i)); // Was giving errors if I dereferenced mutable_output. I thought you had to dereference Mutex guards, but this works now.
            }
        }))
    }
    for handle in handles {
        handle.join().unwrap()
    }
    for output_text in output.lock().unwrap().iter() {
        println!("{}", &output_text)
    }
}
```
#### Output
```
Thread 0 incremented counter
Thread 3 incremented counter
Thread 1 incremented counter
Thread 2 incremented counter
Thread 3 incremented counter
Thread 0 incremented counter
Thread 1 incremented counter
Thread 2 incremented counter
Thread 0 incremented counter
Thread 3 incremented counter
Thread 1 incremented counter
Thread 2 incremented counter
Thread 1 incremented counter
Thread 3 incremented counter
Thread 0 incremented counter
Thread 2 incremented counter
Thread 3 incremented counter
Thread 0 incremented counter
Thread 1 incremented counter
Thread 2 incremented counter
```
This worked great. Most important part of all this stuff is to only keep the lock as long as is absolutely necessary.

## Test 3
Next I need to make a deadlock just to make sure I don't make in the future. This is when two or more threads wait for eachother forever.
### Attempt 1
```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let resource_a = Arc::new(Mutex::new(0));
    let resource_b = Arc::new(Mutex::new(0));

    let a_clone_1 = Arc::clone(&resource_a);
    let b_clone_1 = Arc::clone(&resource_b);
    let thread_1 = thread::spawn(move || {
        println!("Thread 1 locked a");
        let a_guard = a_clone_1.lock().unwrap();
        thread::sleep(Duration::from_secs(1));
        println!("Thread 1 trying to lock b");
        let b_guard = b_clone_1.lock().unwrap();
    });

    let a_clone_2 = Arc::clone(&resource_a);
    let b_clone_2 = Arc::clone(&resource_b);
    let thread_2 = thread::spawn(move || {
        let b_guard = b_clone_2.lock().unwrap();
        println!("Thread 2 locked b");
        thread::sleep(Duration::from_secs(1));
        println!("Thread 2 trying to lock a");
        let a_guard = a_clone_2.lock().unwrap();
    });

    thread_1.join().unwrap();
    thread_2.join().unwrap();
}
```
#### Output
```
Thread 1 locked a
Thread 2 locked b
Thread 1 trying to lock b
Thread 2 trying to lock a
(hangs here forever)
```
Basically thread 1 is trying to lock resource b while thread 2 has it locked, and thread 2 is trying to lock resouce a while thread 1 has it locked. Hence a deadlock.

#### Golden Rule to prevent Deadlocks (IMPORTANT)
**Always acquire locks in the same order.**

## Project
Would be parallel file processor / crawler. However due to my progress Chat doesn't think I need to.
