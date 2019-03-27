use crossbeam;
use std::thread;
use err::HecateError;

pub enum TaskType {
    TileRegen
}

pub struct Task {
    job: TaskType,
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
}

///
/// Main logic for web worker
///
fn worker(rx: crossbeam::Receiver<Task>) {

}

