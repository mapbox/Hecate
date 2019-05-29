use crossbeam;
use postgres;
use std::thread;
use crate::{delta, mvt, webhooks};

pub enum TaskType {
    Delta(i64),
    User(i64),
    Style(i64),
    Meta
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

        match webhooks::send(&conn, &task.job) {
            Err(err) => println!("HecateError: {:?}", &err.to_string()),
            _ => ()
        };

        match task.job {
            TaskType::Delta(delta_id) => {
                let tiles = delta::tiles(&conn, &delta_id).unwrap();

                if tiles.len() == 0 {
                    continue;
                }

                for tile in tiles {
                    if mvt::get(&conn, tile.2, tile.0 as u32, tile.1 as u32, true).is_err() {
                        println!("Daemon: Failed to generate tile: {:?}", tile);
                    }
                }
            },
            _ => ()
        }
    }
}

