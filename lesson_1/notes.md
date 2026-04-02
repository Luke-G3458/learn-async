# Lesson 1: Threads & Ownership Notes
## First test
Creates new thread, which sometimes completes and sometimes doesn't before the main process finishes.
```rust
use std::thread;

fn main() {
    thread::spawn(|| { // <--- New thread, seperate from main
        println!("Hello from thread!");
    });

    println!("Hello from main!"); // <--- Running in main thread
}
```

### Fix
Must use `.join()` in order to require the thread to finish before moving on.
```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| { // <--- Thread
        println!("Hello from thread!");
    });

    handle.join().unwrap(); // <--- Forces thread to finish before moving on

    println!("Hello from main!");
}
```

## Test 2
The `data` variable is in the main thread. The thread may outlive the main thread,
resulting in the thread attempting to access invalid memory. Rust says no bueno.
```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    thread::spawn(|| {
        println!("{:?}", data);
    });
}
```
error: "closure may outlive the current function, but it borrows `data`, which is owned by the current function"

### Fix
Use `move` to give the thread ownership of that data.
```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    let handle = thread::spawn(move || { // <--- transfers ownership of data to this thread
        println!("{:?}", data);
    });

    handle.join().unwrap();
}
```
> Note: If you try to access `data` outside of that thread you will now get errors:
```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    let handle = thread::spawn(move || { // <--- transfers ownership of data to this thread
        println!("{:?}", data);
    });

    handle.join().unwrap();
    println!("{:?}", data);
}
```
error: "borrow of moved value: data"

## Practice problem:
- Create a vector of numbers
- Spawn a thread
- Move the vector into the thread
- Print the sum inside the thread
- Wait for it with join()
### Solution
```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        println!("{:?}", data.iter().sum::<i32>());
    });

    handle.join().unwrap();
}
```

## Project: Parallel Job Runner
### Step 1
```rust
use std::thread;

fn main() {
    let number_of_workers = 4;
    let mut handlers = vec![];
    for i in 0..number_of_workers {
        handlers.push(thread::spawn(move ||{
            println!("Hello from worker {}", i);
        }))
    }
    for i in handlers { i.join().unwrap(); }
}
```
my output: 
```
Hello from worker 2
Hello from worker 0
Hello from worker 3
Hello from worker 1
```
### Step 2
```rust
use std::thread;
use std::time::Duration;

fn main() {
    let number_of_workers = 4;
    let mut handlers = vec![];
    for i in 0..number_of_workers {
        handlers.push(thread::spawn(move || {
            println!("Worker {} started job", i);
            thread::sleep(Duration::from_secs(i + 1));
            println!("Worker {} finished job", i);
        }))
    }
    for i in handlers {
        i.join().unwrap();
    }
}
```
my output:
```
Worker 0 started job
Worker 3 started job
Worker 1 started job
Worker 2 started job
Worker 0 finished job
Worker 1 finished job
Worker 2 finished job
Worker 3 finished job
```

### Step 3: Learning about channels
#### Attempt 1
```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    let work: Vec<_> = (0..10).collect();

    for job in work {
        tx.send(job).unwrap();
    }

    let mut handlers = vec![];

    for i in 0..4 {
        handlers.push(thread::spawn(move || {
            println!("Workder {} started", i);
            loop {
                let job = rx.recv().unwrap();
                println!("Worker {} got job {}", i, job);
                thread::sleep(Duration::from_secs(job));
                println!("Workder {} finished job {}", i, job);
            }
        }))
    }

    for handler in handlers {
        handler.join().unwrap();
    }
}
```
##### Known issues
- ownership of `rx` is moved into closure in threads. This causes obvious issues since there are multiple workers that need to access it.

#### Attempt 2: Use `Arc` to share ownership of `rx` across multiple threads and `Mutex` to ensure that only one thread can access `rx` at a time.
```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

fn main() {
    // Creates a new channel with a transmitter (tx) and a receiver (rx).
    let (tx, rx) = mpsc::channel();

    // Wraps the receiver in an Arc and Mutex. Arc allows multiple threads to share ownership
    // of the receiver, while Mutex ensures that only one thread can access it at a time via locking.
    let rx = Arc::new(Mutex::new(rx));

    let work: Vec<_> = (0..10).collect();

    // Sends jobs to threads through channel via transmitter
    for job in work {
        tx.send(job).unwrap();
    }

    let mut handlers = vec![];

    for i in 0..4 {
        // necessary for each thread to have shared access. Doesn't actually copy the reciever, just
        //shares ownership of it.
        let rx = Arc::clone(&rx);
        
        handlers.push(thread::spawn(move || {
            println!("Worker {} started", i);
            loop {
                // lock and recv must be in one line or else the lock will not
                // be released, and the other threads will have to wait until
                // this process is finished to aquire the lock. Having recv() on
                // the same line results in  the lock being released right after recv()
                // is called.
                match rx.lock().unwrap().recv() {
                    
                    Ok(job) => {
                        println!("Worker {} got job {}", i, job);
                        thread::sleep(Duration::from_secs(job));
                        println!("Worker {} finished job {}", i, job);
                    }
                    Err(_) => break, // This will get triggered when the transmitter is dropped.
                }
            }
        }))
    }

    drop(tx); // Drops the transmitter.

    for handler in handlers {
        handler.join().unwrap();
    }
}
```
##### Output:
```
Worker 0 started
Worker 0 got job 0
Worker 3 started
Worker 2 started
Worker 1 started
Worker 0 finished job 0
Worker 0 got job 1
Worker 0 finished job 1
Worker 0 got job 2
Worker 0 finished job 2
Worker 0 got job 3
Worker 0 finished job 3
Worker 0 got job 4
Worker 0 finished job 4
Worker 0 got job 5
Worker 0 finished job 5
Worker 0 got job 6
Worker 0 finished job 6
Worker 0 got job 7
Worker 0 finished job 7
Worker 0 got job 8
Worker 0 finished job 8
Worker 0 got job 9
Worker 0 finished job 9
```
##### Known issue:
Only worker 0 ever gets a job, resulting in this being effectively single-threaded. I don't fully understand why yet. I would think that the ohter threads could start a job while worker 0 is working on a job. However basically you should never have multiple receivers for the same channel.

### Step 4: implement ThreadPool
#### Attempt 1:
```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{JoinHandle, spawn};

fn main() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        let job = move || {
            println!("Job {} is being processed", i);
        };
        pool.execute(job);
    }
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::new();

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(job)).unwrap();
    }
}

struct Worker {
    id: usize,
    handle: JoinHandle<()>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>>,
    ) -> Self {
        let handle = spawn(move || {
            loop {
                match receiver.lock().unwrap().recv() {
                    Ok(job) => {
                        println!("Worker {} got a job; executing", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });
        Worker { id, handle }
    }
}
```
##### Output
```
Worker 0 got a job; executing
Job 0 is being processed
Worker 0
```
##### Known Issues
main process ends before anything much can happen.

#### Attempt 2:
```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{JoinHandle, spawn};

fn main() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        let job = move || {
            println!("Job {} is being processed", i);
        };
        pool.execute(job);
    }
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::new();

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            };
        }
    }
}

struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>>,
    ) -> Self {
        let handle = spawn(move || {
            loop {
                match receiver.lock().unwrap().recv() {
                    Ok(job) => {
                        println!("Worker {} got a job; executing", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });
        Worker {
            id,
            handle: Some(handle),
        }
    }
}
```
##### Output
Worker 1 got a job; executing
Job 0 is being processed
Worker 1 got a job; executing
Job 1 is being processed
Worker 1 got a job; executing
Job 2 is being processed
Worker 1 got a job; executing
Job 3 is being processed
Worker 1 got a job; executing
Job 4 is being processed
Worker 1 got a job; executing
Job 5 is being processed
Worker 1 got a job; executing
Job 6 is being processed
Worker 1 got a job; executing
Job 7 is being processed
Worker 1 shutting down
Worker 3 shutting down
Worker 2 shutting down
Worker 0 shutting down
(Note: This all printed almost immediately)
##### Known Issues:
- Only worker 1 is doing all the work, due to the fact that the jobs are almost instantaneous, so worker 1 is able to get a job, finish a job, and pick up the next job immediately. Same issue as before ThreadPool.

#### Attempt 3: Add a sleep to the job to allow for other workers to pick up jobs.
```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{JoinHandle, spawn};

fn main() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        let job = move || {
            println!("Job {} is being processed", i);
            std::thread::sleep(std::time::Duration::from_secs(1)); // Delay added to see if other workers are included.
            println!("Job {} finished", i)
        };
        pool.execute(job);
    }
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::new();

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            sender.send(Box::new(job)).unwrap();
        }
    }
}

// When the ThreadPool goes out of scope, it dops the sender and then joins all threads.
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            };
        }
    }
}

struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>>,
    ) -> Self {
        let handle = spawn(move || {
            loop {
                match receiver.lock().unwrap().recv() {
                    Ok(job) => {
                        println!("Worker {} got a job; executing", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });
        Worker {
            id,
            handle: Some(handle),
        }
    }
}
```
##### Output
```
Worker 0 got a job; executing
Job 0 is being processed
Job 0 finished
Worker 0 got a job; executing
Job 1 is being processed
Job 1 finished
Worker 0 got a job; executing
Job 2 is being processed
Job 2 finished
Worker 0 got a job; executing
Job 3 is being processed
Job 3 finished
Worker 0 got a job; executing
Job 4 is being processed
Job 4 finished
Worker 0 got a job; executing
Job 5 is being processed
Job 5 finished
Worker 0 got a job; executing
Job 6 is being processed
Job 6 finished
Worker 0 got a job; executing
Job 7 is being processed
Job 7 finished
Worker 0 shutting down
Worker 3 shutting down
Worker 2 shutting down
Worker 1 shutting down
```
##### Known Issues:
- Still only one worker is doing all the work. Caused by other threads trying to get a lock on receiver and getting blocked and then not continuing to

## Project 2 (all me)
### Goal
To create a single thread which manages multiple threads. The main thread sends jobs via channel to the manager thread, which then sends the jobs to worker threads.
### Reason
The previous ThreadPool design was not parallel due to sharing (Mutex) the receiver across all worker threads. This design allows for each worker thread to have its own receiver, so they can all work in parallel.
### Attempt 1
> main.rs
```rust
use lesson_1::manager::Manager;
use std::thread;
use std::time::Duration;

fn main() {
    let manager = Manager::new(4);
    for i in 0..10 {
        manager.assign_job(move || {
            println!("Job {} is running", i);
            thread::sleep(Duration::from_secs(1));
            println!("job {} is done", i);
        });
        manager.tick();
    }
}
```

> manager.rs
```rust
use crate::worker::Worker;
use std::sync::mpsc;

pub struct JobFinished {
    worker_id: usize,
}

pub struct Manager {
    workers: Vec<Worker>,
    feedback_receiver: std::sync::mpsc::Receiver<JobFinished>,
}

impl Manager {
    pub fn new(num_workers: usize) -> Self {
        let mut workers = Vec::new();
        let (sender, receiver) = mpsc::channel();
        for id in 0..num_workers {
            let (assignment_sender, receiver) = mpsc::channel();
            workers.push(Worker::new(id, receiver, assignment_sender, sender.clone()));
        }
        Self {
            workers,
            feedback_receiver: receiver,
        }
    }

    pub fn tick(&self) {
        if let Ok(job_finished) = self.feedback_receiver.recv() {
            println!(
                "Manager received job finished from Worker {}",
                job_finished.worker_id
            );
            for worker in &self.workers {
                if worker.id == job_finished.worker_id {
                    let mut busy = worker.busy.lock().unwrap();
                    *busy = false;
                    break;
                }
            }
        }
    }

    pub fn assign_job<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        for worker in &self.workers {
            let mut busy = worker.busy.lock().unwrap();
            if !*busy {
                *busy = true;
                drop(busy);
                if let Some(sender) = &worker.assignment_sender {
                    sender.send(Box::new(job)).unwrap();
                    break;
                }
            }
        }
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(sender) = worker.assignment_sender.take() {
                drop(sender)
            }
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
```

> worker.rs
```rust
use crate::manager::JobFinished;

pub struct Worker {
    pub id: usize,
    pub handle: Option<std::thread::JoinHandle<()>>,
    pub assignment_sender: Option<std::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>>,
    pub feedback_sender: Option<std::sync::mpsc::Sender<JobFinished>>,
    pub busy: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl Worker {
    pub fn new(
        id: usize,
        receiver: std::sync::mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>,
        assignment_sender: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
        feedback_sender: std::sync::mpsc::Sender<JobFinished>,
    ) -> Self {
        let handle = std::thread::spawn(move || {
            println!("Worker {} starting", id);
            loop {
                match receiver.recv() {
                    Ok(job) => {
                        println!("Worker {} receiver job", id);
                        job();
                        println!("Worker {} finished job", id);
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });
        Self {
            id,
            handle: Some(handle),
            assignment_sender: Some(assignment_sender),
            feedback_sender: Some(feedback_sender),
            busy: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }
}
```
##### Output
```
Worker 2 starting
Worker 0 starting
Worker 0 receiver job
Job 0 is running
Worker 1 starting
Worker 3 starting
job 0 is done
Worker 0 finished job
(Note: just sat after this)
```
There are a couple things wrong with this. See comments on code for specific issues. Overall, the manager sends one job and then at the `.tick()` method just sits at the `.recv()` until the worker finishes the job and sends the feedback. However, the worker currently doesn't even have a way to send feedback after completing a job (🤦‍♂️). However I have no way to access the `Worker` struct's `feedback_sender` from the job closure... The reason I have a feedback sender in the first place is because I have a `busy` field in the `Worker` struct, but I was unable to set that from the job closure. However the feedback_sender cannot be accessed from the job closure either.

##### Solution Option 1:
Rather than having the `feedback_sender` as a field in the `Worker` struct, I will pass it directly to the job closure, allowing it to send feedback to the manager after completing a job.

##### Solution Option 2:
Rather than having the worker send feedback to the manager, give the job closure an Arc::clone() of the worker's busy field, so that the job can set the worker to not busy after it is done and still allow the manager to check if the worker is busy.
> This requires stripping out some more code, so I'll do this second.

### Attempt 2: Implementing Attempt 1 Solution Option 1
> main.rs
```rust
use lesson_1::manager::Manager;
use std::thread;
use std::time::Duration;

fn main() {
    let manager = Manager::new(4);
    for i in 0..10 {
        manager.assign_job(move || {
            println!("Job {} is running", i);
            thread::sleep(Duration::from_secs(1));
            println!("job {} is done", i);
        });
        manager.tick();
    }
}
```

> manager.rs
```rust
use crate::worker::Worker;
use std::sync::mpsc;

pub struct JobFinished {
    pub worker_id: usize,
}

pub struct Manager {
    workers: Vec<Worker>,
    feedback_receiver: std::sync::mpsc::Receiver<JobFinished>,
}

impl Manager {
    pub fn new(num_workers: usize) -> Self {
        let mut workers = Vec::new();
        let (sender, receiver) = mpsc::channel();
        for id in 0..num_workers {
            let (assignment_sender, receiver) = mpsc::channel();
            workers.push(Worker::new(id, receiver, assignment_sender, sender.clone()));
        }
        Self {
            workers,
            feedback_receiver: receiver,
        }
    }

    pub fn tick(&self) {
        if let Ok(job_finished) = self.feedback_receiver.recv() {
            println!(
                "Manager received job finished from Worker {}",
                job_finished.worker_id
            );
            for worker in &self.workers {
                if worker.id == job_finished.worker_id {
                    let mut busy = worker.busy.lock().unwrap();
                    *busy = false;
                    break;
                }
            }
        }
    }

    pub fn assign_job<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        for worker in &self.workers {
            let mut busy = worker.busy.lock().unwrap();
            if !*busy {
                *busy = true;
                drop(busy);
                if let Some(sender) = &worker.assignment_sender {
                    sender.send(Box::new(job)).unwrap();
                    break;
                }
            }
        }
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(sender) = worker.assignment_sender.take() {
                drop(sender)
            }
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
```

> worker.rs
```rust
use crate::manager::JobFinished;

pub struct Worker {
    pub id: usize,
    pub handle: Option<std::thread::JoinHandle<()>>,
    pub assignment_sender: Option<std::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>>,
    pub busy: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl Worker {
    pub fn new(
        id: usize,
        receiver: std::sync::mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>,
        assignment_sender: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
        feedback_sender: std::sync::mpsc::Sender<JobFinished>,
    ) -> Self {
        let handle = std::thread::spawn(move || {
            println!("Worker {} starting", id);
            loop {
                match receiver.recv() {
                    Ok(job) => {
                        println!("Worker {} receiver job", id);
                        job();
                        println!("Worker {} finished job", id);
                        feedback_sender.send(JobFinished { worker_id: id }).unwrap();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });
        Self {
            id,
            handle: Some(handle),
            assignment_sender: Some(assignment_sender),
            busy: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }
}
```
#### Output
```
Worker 3 starting
Worker 0 starting
Worker 0 receiver job
Worker 2 starting
Worker 1 starting
Job 0 is running
job 0 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 1 is running
job 1 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 2 is running
job 2 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 3 is running
job 3 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 4 is running
job 4 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 5 is running
job 5 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 6 is running
job 6 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 7 is running
job 7 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 8 is running
job 8 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 receiver job
Job 9 is running
job 9 is done
Worker 0 finished job
Manager received job finished from Worker 0
Worker 0 shutting down
Worker 1 shutting down
Worker 2 shutting down
Worker 3 shutting down
```
Well I fixed the original issue of the worker not being able to send a feedback message. However there is still an issue now because the `.tick()` method causes the manager just to sit at the `.recv()` until a worker finishes. Then it sends another job only after the previous job is finished. Rather than trying attempt 1 solution option 2 which feels a little janky, another solution would be to have the main thread assign all the jobs at once and then loop through `.tick()` as many times as jobs that were assigned.
### Attempt 3: Modifying `main.rs`
> main.rs
```rust
use lesson_1::manager::Manager;
use std::thread;
use std::time::Duration;

fn main() {
    let manager = Manager::new(4);
    for i in 0..10 {
        manager.assign_job(move || {
            println!("Job {} is running", i);
            thread::sleep(Duration::from_secs(1));
            println!("job {} is done", i);
        });
    }
    for _ in 0..10 {
        manager.tick();
    }
}
```
#### Output
```
Worker 3 starting
Worker 3 receiver job
Worker 1 starting
Worker 1 receiver job
Job 1 is running
Worker 2 starting
Worker 2 receiver job
Job 2 is running
Job 3 is running
Worker 0 starting
Worker 0 receiver job
Job 0 is running
job 1 is done
Worker 1 finished job
Manager received job finished from Worker 1
job 3 is done
Worker 3 finished job
job 2 is done
Worker 2 finished job
job 0 is done
Worker 0 finished job
Manager received job finished from Worker 3
Manager received job finished from Worker 2
Manager received job finished from Worker 0
(Note: just sat here)
```
Closer, but still not great. Some issues:
- Manager doesn't store jobs, actually assigns them. If there is no worker available it just forgets about that. Gotta fix this.

### Attempt 3
> main.rs
```rust
use lesson_1::manager::Manager;
use std::thread;
use std::time::Duration;

fn main() {
    let mut manager = Manager::new(4);
    for i in 0..10 {
        manager.schedule_job(Box::new(move || {
            println!("Job {} is running", i);
            thread::sleep(Duration::from_secs(1));
            println!("job {} is done", i);
        }));
    }
    for _ in 0..10 {
        manager.tick();
    }
}
```

> manager.rs
```rust
use crate::worker::Worker;
use std::sync::mpsc;

pub struct JobFinished {
    pub worker_id: usize,
}

pub struct Manager {
    workers: Vec<Worker>,
    feedback_receiver: std::sync::mpsc::Receiver<JobFinished>,
    jobs: Vec<Box<dyn FnOnce() + Send + 'static>>,
}

impl Manager {
    pub fn new(num_workers: usize) -> Self {
        let mut workers = Vec::new();
        let (sender, receiver) = mpsc::channel();
        for id in 0..num_workers {
            let (assignment_sender, receiver) = mpsc::channel();
            workers.push(Worker::new(id, receiver, assignment_sender, sender.clone()));
        }
        Self {
            workers,
            feedback_receiver: receiver,
            jobs: vec![],
        }
    }

    pub fn get_available_workers(&self) -> Option<Vec<usize>> {
        let mut output = Vec::new();
        for worker in &self.workers {
            if worker.busy {
                continue;
            } else {
                output.push(worker.id);
            }
        }
        if output.len() < 1 {
            return None;
        } else {
            return Some(output);
        }
    }

    pub fn tick(&mut self) {
        if let Ok(job_finished) = self.feedback_receiver.recv() {
            println!(
                "Manager received job finished from Worker {}",
                job_finished.worker_id
            );
            for worker in &mut self.workers {
                if worker.id == job_finished.worker_id {
                    worker.busy = false;
                    break;
                }
            }
        }
        while let Some(job) = self.jobs.pop() {
            if let Some(available_workers) = self.get_available_workers() {
                if available_workers.len() > 0 {
                    self.schedule_job(job);
                    continue;
                }
            }
            self.jobs.push(job);
            break;
        }
    }

    pub fn schedule_job(&mut self, job: Box<dyn FnOnce() + Send + 'static>) {
        if let Some(available_workers) = self.get_available_workers() {
            for available_worker_id in available_workers {
                self.assign_job(available_worker_id, job);
                return;
            }
        }
        self.jobs.push(job);
    }

    pub fn assign_job(&mut self, worker_id: usize, job: Box<dyn FnOnce() + Send + 'static>) {
        for worker in &mut self.workers {
            if worker.id == worker_id {
                worker.busy = true;
                if let Some(sender) = &worker.assignment_sender {
                    sender.send(job).unwrap();
                    break;
                }
            }
        }
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(sender) = worker.assignment_sender.take() {
                drop(sender)
            }
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
```

> worker.rs
```rust
use crate::manager::JobFinished;

pub struct Worker {
    pub id: usize,
    pub handle: Option<std::thread::JoinHandle<()>>,
    pub assignment_sender: Option<std::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>>,
    pub busy: bool,
}

impl Worker {
    pub fn new(
        id: usize,
        receiver: std::sync::mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>,
        assignment_sender: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
        feedback_sender: std::sync::mpsc::Sender<JobFinished>,
    ) -> Self {
        let handle = std::thread::spawn(move || {
            println!("Worker {} starting", id);
            loop {
                match receiver.recv() {
                    Ok(job) => {
                        println!("Worker {} received job", id);
                        job();
                        println!("Worker {} finished job", id);
                        feedback_sender.send(JobFinished { worker_id: id }).unwrap();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
        });
        Self {
            id,
            handle: Some(handle),
            assignment_sender: Some(assignment_sender),
            busy: false,
        }
    }
}
```

#### Output
```
Worker 2 starting
Worker 2 received job
Job 2 is running
Worker 1 starting
Worker 1 received job
Job 1 is running
Worker 0 starting
Worker 0 received job
Job 0 is running
Worker 3 starting
Worker 3 received job
Job 3 is running
job 2 is done
Worker 2 finished job
Manager received job finished from Worker 2
Worker 2 received job
Job 9 is running
job 0 is done
Worker 0 finished job
job 3 is done
Worker 3 finished job
Manager received job finished from Worker 0
Manager received job finished from Worker 3
Worker 0 received job
Job 8 is running
Worker 3 received job
Job 7 is running
job 1 is done
Worker 1 finished job
Manager received job finished from Worker 1
Worker 1 received job
Job 6 is running
job 9 is done
Worker 2 finished job
Manager received job finished from Worker 2
Worker 2 received job
Job 5 is running
job 8 is done
job 7 is done
Worker 3 finished job
Worker 0 finished job
Manager received job finished from Worker 3
Manager received job finished from Worker 0
Worker 3 received job
Job 4 is running
job 6 is done
Worker 1 finished job
Manager received job finished from Worker 1
job 5 is done
Worker 2 finished job
Manager received job finished from Worker 2
job 4 is done
Worker 3 finished job
Manager received job finished from Worker 3
Worker 0 shutting down
Worker 1 shutting down
Worker 2 shutting down
Worker 3 shutting down
```
Oh ya. That's what I'm talking about. Took be 2 hours, but check that out. Next step is just to optimize it.
