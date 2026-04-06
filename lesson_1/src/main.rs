use lesson_1::manager::Manager;
use std::thread;
use std::time::Duration;

fn main() {
    let mut manager = Manager::start(4);
    for i in 0..10 {
        manager.schedule(Box::new(move || {
            println!("Job {} is running", i);
            thread::sleep(Duration::from_secs(1));
            println!("job {} is done", i);
        }));
    }
}
