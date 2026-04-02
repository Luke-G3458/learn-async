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
