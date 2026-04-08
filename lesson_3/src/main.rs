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
