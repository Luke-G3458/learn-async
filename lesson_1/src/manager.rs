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
