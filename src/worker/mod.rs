use crossbeam;
use std::thread;
use err::HecateError;

pub enum TaskType {
    Delta(i64)
}

pub struct Task {
    job: TaskType
}

impl Task {
    pub fn new(tasktype: TaskType) -> Self {
        Task {
            job: tasktype
        }
    }
}

pub struct Worker {
    sender: crossbeam::Sender<Task>
}

impl Worker {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam::channel::unbounded();

        let thread = thread::Builder::new().name(String::from("Hecate Daemon")).spawn(move || {
            worker(rx);
        }).unwrap();

        Worker {
            sender: tx,
        }
    }

    pub fn queue(&self, task: Task) {
        self.sender.send(task);
    }
}

///
/// Main logic for web worker
///
fn worker(rx: crossbeam::Receiver<Task>) {

}

