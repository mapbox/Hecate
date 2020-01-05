use crate::err::HecateError;

use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;

use rand::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub struct Database {
    pub main: String,
    pub replica: Vec<String>,
    pub sandbox: Vec<String>
}

impl Database {
    pub fn new(main: String, replica: Vec<String>, sandbox: Vec<String>) -> Self {
        Database {
            main,
            replica,
            sandbox
        }
    }
}

pub type PostgresPool = Pool<PostgresConnectionManager<postgres::Client>>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager<postgres::Client>>;

pub fn init_pool(database: &str) -> r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::Client>> {
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(format!("postgres://{}", database), postgres::NoTls).unwrap();
    match r2d2::Pool::builder().max_size(15).build(manager) {
        Ok(pool) => pool,
        Err(_) => {
            println!("ERROR: Failed to connect to database");
            std::process::exit(1);
        }
    }
}

#[derive(Clone)]
pub struct DbReplica(pub Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::Client>>>>);
impl DbReplica {
    pub fn new(database: Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::Client>>>>) -> Self {
        DbReplica(database)
    }

    pub fn get(&self) -> Result<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager<postgres::Client>>, HecateError> {
        match self.0 {
            None => Err(HecateError::new(503, String::from("No Database Replica Connection"), None)),
            Some(ref db_replica) => {
                let mut rng = thread_rng();
                let db_replica_it = rng.gen_range(0, db_replica.len());

                match db_replica.get(db_replica_it).unwrap().get() {
                    Ok(conn) => Ok(conn),
                    Err(_) => Err(HecateError::new(503, String::from("Could not connect to database"), None))
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct DbSandbox(pub Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::Client>>>>);
impl DbSandbox {
    pub fn new(database: Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::Client>>>>) -> Self {
        DbSandbox(database)
    }

    pub fn get(&self) -> Result<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager<postgres::Client>>, HecateError> {
        match self.0 {
            None => Err(HecateError::new(503, String::from("No Database Sandbox Connection"), None)),
            Some(ref db_sandbox) => {
                let mut rng = thread_rng();
                let db_sandbox_it = rng.gen_range(0, db_sandbox.len());

                match db_sandbox.get(db_sandbox_it).unwrap().get() {
                    Ok(conn) => Ok(conn),
                    Err(_) => Err(HecateError::new(503, String::from("Could not connect to database"), None))
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct DbReadWrite(pub r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::Client>>); //Read & Write DB Connection
impl DbReadWrite {
    pub fn new(database: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::Client>>) -> Self {
        DbReadWrite(database)
    }

    pub fn get(&self) -> Result<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager<postgres::Client>>, HecateError> {
        match self.0.get() {
            Ok(conn) => Ok(conn),
            Err(_) => Err(HecateError::new(503, String::from("Could not connect to database"), None))
        }
    }
}
