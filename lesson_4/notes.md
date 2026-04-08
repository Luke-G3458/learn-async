# Lesson 4: Async Fundamentals
## Notes
### What is the difference between threading and Async?
In threading if a thread sleeps then the OS switches to another thread. In Async if a task is not ready then the executor (part of the async runtime) will run a different task.
### Core Abstraction
`Future` = a value that may not be ready yet.  
A future is sort of like an enum:
```rust
enum FutureState {
    Ready(T),
    Pending,
}
```
The executor does this:
```
loop:
    poll(task)
    if Ready -> done
    if Pending -> try later
```
## Test 1: Fake Async System
Goal: 
- Tasks that take multiple “ticks” to complete
- An executor that keeps polling them
All single threaded for now.
### Things to make:
- Task: contains and id and a counter that can be decremented each time it is polled. Should also have a poll method which decrements the counter, prints progress, and then returns Pending if not done or Ready if done.
- Poll Result: an enum that can be either Pending or Ready.
- Executor: something that stores multiple tasks and repeatedly iterates over them, polls each one, and removes completed ones.
### Attempt 1
```rust
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
```
#### Output
```
Job 3 counter at 9
Job 2 counter at 1
Job 1 counter at 4
Job 3 counter at 8
Job 2 finished
Job 1 counter at 3
Job 3 counter at 7
Job 1 counter at 2
Job 3 counter at 6
Job 1 counter at 1
Job 3 counter at 5
Job 1 finished
Job 3 counter at 4
Job 3 counter at 3
Job 3 counter at 2
Job 3 counter at 1
Job 3 finished
```
