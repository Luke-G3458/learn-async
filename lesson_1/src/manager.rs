use crate::worker::Worker;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

pub struct JobFinished {
    pub worker_id: usize,
}

pub type Job = Box<dyn FnOnce() + Send + 'static>;

pub enum Message {
    AssignJob(Job),
    WorkerFinished(JobFinished),
    Close,
}

pub struct Manager {
    main_sender: Sender<Message>,
    handle: Option<JoinHandle<()>>,
}

impl Manager {
    pub fn start(num_workers: usize) -> Self {
        let (main_sender, main_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
        let cloned_sender = main_sender.clone();
        let handle = thread::spawn(move || {
            let mut manager = ManagerThread::new(num_workers, cloned_sender.clone());
            manager.run(main_receiver);
        });
        Self {
            main_sender,
            handle: Some(handle),
        }
    }

    pub fn schedule(&self, job: Job) {
        self.main_sender.send(Message::AssignJob(job)).unwrap();
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        self.main_sender.send(Message::Close).unwrap();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        };
    }
}

pub struct ManagerThread {
    workers: Vec<Worker>,
    available_workers: VecDeque<usize>,
    jobs: VecDeque<Job>,
}

impl ManagerThread {
    pub fn new(num_workers: usize, sender: Sender<Message>) -> Self {
        let mut workers = Vec::with_capacity(num_workers);
        let mut available_workers = VecDeque::with_capacity(num_workers);
        for id in 0..num_workers {
            let (assignment_sender, receiver) = mpsc::channel();
            workers.push(Worker::new(id, receiver, assignment_sender, sender.clone()));
            available_workers.push_back(id);
        }
        Self {
            workers,
            available_workers,
            jobs: VecDeque::new(),
        }
    }

    pub fn run(&mut self, receiver: Receiver<Message>) {
        let mut shutting_down = false;

        loop {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    Message::AssignJob(job) => self.jobs.push_back(job),
                    Message::WorkerFinished(feedback) => {
                        self.available_workers.push_back(feedback.worker_id)
                    }
                    Message::Close => shutting_down = true,
                }
            }

            while !self.jobs.is_empty() && !self.available_workers.is_empty() {
                let job = self.jobs.pop_front().unwrap();
                let worker_id = self.available_workers.pop_front().unwrap();

                self.assign_job(worker_id, job);
            }

            if shutting_down
                && self.jobs.is_empty()
                && self.available_workers.len() == self.workers.len()
            {
                break;
            }

            if self.jobs.is_empty() || self.available_workers.is_empty() {
                match receiver.recv() {
                    Ok(msg) => match msg {
                        Message::AssignJob(job) => self.jobs.push_back(job),
                        Message::Close => shutting_down = true,
                        Message::WorkerFinished(feedback) => {
                            self.available_workers.push_back(feedback.worker_id)
                        }
                    },
                    Err(_) => shutting_down = true,
                }
            }

            if shutting_down
                && self.jobs.is_empty()
                && self.available_workers.len() == self.workers.len()
            {
                break;
            }
        }
    }

    pub fn assign_job(&mut self, worker_id: usize, job: Job) {
        let worker = &self.workers[worker_id];
        if let Some(sender) = &worker.assignment_sender {
            sender.send(job).unwrap();
        }
    }
}

impl Drop for ManagerThread {
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
