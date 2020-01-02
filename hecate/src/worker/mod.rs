use crossbeam;
use postgres;
use std::thread;
use crate::{delta, mvt, webhooks};

#[derive(Debug,PartialEq)]
pub enum TaskType {
    Delta(i64),
    User(String),
    Style(i64),
    Meta
}

#[derive(Debug,PartialEq)]
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

#[derive(Clone)]
pub struct Worker {
    sender: crossbeam::Sender<Task>
}

impl Worker {
    pub fn new(database: String) -> Self {
        let (tx, rx) = crossbeam::channel::unbounded();

        thread::Builder::new().name(String::from("Hecate Daemon")).spawn(move || {
            worker(rx, database);
        }).unwrap();

        Worker {
            sender: tx
        }
    }

    pub fn queue(&self, task: Task) {
        if self.sender.send(task).is_err() {
            println!("WARN: Failed to write task to queue");
        }
    }
}

///
/// Main logic for web worker
///
fn worker(rx: crossbeam::Receiver<Task>, database: String) {
    let conn = postgres::Connection::connect(format!("postgres://{}", database), postgres::TlsMode::None).unwrap();

    loop {
        let task = match rx.recv() {
            Ok(task) => task,
            Err(err) => {
                println!("Daemon: Failed to obtain task: {}", err);
                continue;
            }
        };

        if let Err(err) = webhooks::send(&conn, &task.job) {
            println!("HecateError: {:?}", &err.to_string());
        }

        if let TaskType::Delta(delta_id) = task.job {
            let tiles = delta::tiles(&conn, delta_id, 14, 17).unwrap();

            if tiles.is_empty() {
                continue;
            }

            for tile in tiles {
                if mvt::regen(&conn, tile.2, tile.0 as u32, tile.1 as u32).is_some() {
                    println!("Daemon: Failed to generate tile: {:?}", tile);
                }
            }
        }
    }
}

