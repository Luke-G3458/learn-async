# Lesson 3: Message Passing vs Shared Memory
## Tools
- `sync_channel`: works similar to the mpsc channel, but is bounded with a fixed capacity, and the sender will block if the channel is full until space becomes available. This prevents unbounded memory usage and allows for backpressure in the system. Usage:
```rust
let (tx, rx) = sync_channel(5); // 5 is the queue capacity/size
```

## Test 1
1. Create a bounded channel using `sync_channel`
2. Spawn 3 worker threads, each of which
  - receive jobs
  - sleep (simulate work)
  - prints job info
3. Producer (main thread) which sends 20 jobs (quickly) and print when sending.
**Constraints:**
- Do NOT use Mutex here, except to share the receiver
- Only channels
- Workers loop on recv()
**Goals:** See the sending prints stop for a little while the channel queue is full, then it will resume later.
### Attempt 1
```rust
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub type Job = Box<dyn FnOnce() + Send + 'static>;

fn main() {
    let (tx, rx): (SyncSender<Job>, Receiver<Job>) = sync_channel(5);
    let rx = Arc::new(Mutex::new(rx));
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..3 {
        let rx_thread = Arc::clone(&rx);
        handles.push(thread::spawn(move || {
            println!("Worker {} starting", i);
            loop {
                let guard = rx_thread.lock().unwrap();
                match guard.recv() {
                    Ok(job) => {
                        drop(guard);
                        println!("Worker {} received job", i);
                        job();
                    }
                    Err(_) => {
                        drop(guard);
                        println!("Worker {} exiting", i);
                        break;
                    }
                }
            }
        }));
    }
    for i in 0..20 {
        println!("Sending job {}", i);
        tx.send(Box::new(move || {
            thread::sleep(Duration::from_secs(2));
            println!("Job {} is done", i);
        }))
        .unwrap();
    }

    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }
}
```
#### Output
```
Sending job 0
Sending job 1
Worker 2 starting
Worker 2 received job
Sending job 2
Sending job 3
Sending job 4           // Lots of jobs sent here...
Sending job 5
Sending job 6           // but then stops here due to the channel queue being full. This is called backpressure, 
Worker 1 starting       // which is when the producer is slowed down by the consumer's ability to process jobs.
Worker 0 starting
Worker 0 received job
Sending job 7
Sending job 8
Worker 1 received job
Job 0 is done
Worker 2 received job
Job 2 is done
Worker 0 received job
Job 1 is done
Worker 1 received job
Sending job 9
Sending job 10
Sending job 11
Job 3 is done
Worker 2 received job
Job 4 is done
Worker 0 received job
Job 5 is done
Worker 1 received job
Sending job 12
Sending job 13
Sending job 14
Job 6 is done
Worker 2 received job
Sending job 15
Job 7 is done
Worker 0 received job
Sending job 16
Job 8 is done
Worker 1 received job
Sending job 17
Job 9 is done
Worker 2 received job
Sending job 18
Job 10 is done
Worker 0 received job
Sending job 19
Job 11 is done
Worker 1 received job
Job 12 is done
Worker 2 received job
Job 13 is done
Worker 0 received job
Job 14 is done
Worker 1 received job
Job 15 is done
Worker 2 received job
Job 16 is done
Worker 0 received job
Job 17 is done
Worker 1 exiting
Job 18 is done
Worker 2 exiting
Job 19 is done
Worker 0 exiting
```
### Attempt 2: Quick update to avoid locking the receiver
modify loop in worker threads like so
```rust
loop {
    let job = {
        let guard = rx_thread.lock().unwrap();
        guard.recv()
    };
    match job {
        Ok(job) => {
            println!("Worker {} received job", i);
            job();
        }
        Err(_) => {
            println!("Worker {} exiting", i);
            break;
        }
    }
}
```
## Test 2
Modify previous to instead just drop the jobs if the queue is full by using `try_send` instead of `send`. This is obviously **worse**, but just for testing sake.
### Main concept
- `send`: Safe, but can slow producer
- `try_send`: Fast, but will lose work
### Attempt 1
```rust
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub type Job = Box<dyn FnOnce() + Send + 'static>;

fn main() {
    let (tx, rx): (SyncSender<Job>, Receiver<Job>) = sync_channel(5);
    let rx = Arc::new(Mutex::new(rx));
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..3 {
        let rx_thread = Arc::clone(&rx);
        handles.push(thread::spawn(move || {
            println!("Worker {} starting", i);
            loop {
                let job = {
                    let guard = rx_thread.lock().unwrap();
                    guard.recv()
                };
                match job {
                    Ok(job) => {
                        println!("Worker {} received job", i);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} exiting", i);
                        break;
                    }
                }
            }
        }));
    }
    for i in 0..20 {
        println!("Sending job {}", i);
        match tx.try_send(Box::new(move || {
            thread::sleep(Duration::from_secs(2));
            println!("Job {} is done", i);
        })) {
            Ok(_) => println!("Job {} sent", i),
            Err(_) => println!("Dropped job {}", i),
        }
    }

    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }
}
```
### Output
```
Sending job 0
Worker 0 starting
Worker 0 received job
Job 0 sent
Sending job 1
Job 1 sent
Sending job 2
Job 2 sent
Sending job 3
Worker 2 starting
Worker 2 received job
Job 3 sent
Sending job 4
Job 4 sent
Sending job 5
Job 5 sent
Sending job 6
Job 6 sent
Sending job 7
Dropped job 7
Sending job 8
Dropped job 8
Sending job 9
Dropped job 9
Sending job 10
Dropped job 10
Sending job 11
Dropped job 11
Sending job 12
Dropped job 12
Sending job 13
Dropped job 13
Sending job 14
Dropped job 14
Sending job 15
Dropped job 15
Sending job 16
Dropped job 16
Sending job 17
Dropped job 17
Sending job 18
Dropped job 18
Sending job 19
Dropped job 19
Worker 1 starting
Worker 1 received job
Job 0 is done
Worker 0 received job
Job 1 is done
Worker 2 received job
Job 2 is done
Worker 1 received job
Job 3 is done
Worker 0 received job
Job 4 is done
Worker 2 exiting
Job 5 is done
Worker 1 exiting
Job 6 is done
Worker 0 exiting
```
As expected, it just drops jobs when the channel queue is full. This allows fast throughput, but at the cost of losing work. It is a comparisson between correctness and throughput. In some cases, like logging, it may be acceptable to drop logs if the system is under heavy load, but in other cases, like processing financial transactions, it would be unacceptable to drop jobs.

## Key takeaways
If I ever need to make sure that my memory doesn't blow up I can prevent the producer from sending too much by using a bounded channel called `sync_channel`. There are two options for what this bounded channel can do when the queue is full: it can either block the producer until there is space in the queue, or it can drop the job or do something else with it. Both are useful, but in different scenarios.
