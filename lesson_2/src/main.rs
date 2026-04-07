// Intentional Deadlock program
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
