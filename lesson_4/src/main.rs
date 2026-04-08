pub enum PollResult {
    Ready,
    Pending,
}

pub struct Task {
    id: usize,
    counter: usize,
}

impl Task {
    pub fn new(id: usize, counter_start: usize) -> Self {
        Task {
            id,
            counter: counter_start,
        }
    }
    pub fn poll(&mut self) -> PollResult {
        self.counter -= 1;
        if self.counter == 0 {
            PollResult::Ready
        } else {
            PollResult::Pending
        }
    }
}

pub struct Executor {
    tasks: Vec<Task>,
}

impl Executor {
    pub fn new(tasks: Vec<Task>) -> Self {
        Executor { tasks }
    }
    pub fn run(&mut self) {
        loop {
            for i in (0..self.tasks.len()).rev() {
                match self.tasks[i].poll() {
                    PollResult::Pending => {
                        println!(
                            "Job {} counter at {}",
                            self.tasks[i].id, self.tasks[i].counter
                        );
                        continue;
                    }
                    PollResult::Ready => {
                        println!("Job {} finished", self.tasks[i].id);
                        self.tasks.remove(i);
                    }
                };
            }
            if self.tasks.is_empty() {
                break;
            }
        }
    }
}

fn main() {
    let mut executor = Executor::new(vec![Task::new(1, 5), Task::new(2, 2), Task::new(3, 10)]);
    executor.run()
}
