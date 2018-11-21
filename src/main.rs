extern crate hecate;
#[macro_use] extern crate clap;
extern crate serde_json;
extern crate postgres;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use hecate::auth::CustomAuth;
use std::error::Error;
use clap::App;

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let matched = App::from_yaml(cli_cnf).get_matches();

    let database = String::from(matched.value_of("database").unwrap_or("hecate@localhost:5432/hecate"));

    let database_read = match matched.values_of("database_read") {
        None => vec![String::from("hecate_read@localhost:5432/hecate")],
        Some(db_read) => db_read.map(|db| String::from(db)).collect()
    };

    let schema: Option<serde_json::value::Value> = match matched.value_of("schema") {
        Some(schema_path) => {
            let mut schema_file = match File::open(&Path::new(schema_path)) {
                Ok(file) => file,
                Err(_) => panic!("Failed to open schema file at: {}", schema_path)
            };

            let mut schema_str = String::new();

            schema_file.read_to_string(&mut schema_str).unwrap();

            let schema_json: serde_json::value::Value = serde_json::from_str(&schema_str).unwrap();

            Some(schema_json)
        },
        None => None
    };

    let auth: Option<CustomAuth> = match matched.value_of("auth") {
        Some(auth_path) => {
            let mut auth_file = match File::open(&Path::new(auth_path)) {
                Ok(file) => file,
                Err(_) => panic!("Failed to open auth file at: {}", auth_path)
            };

            let mut auth_str = String::new();

            auth_file.read_to_string(&mut auth_str).unwrap();

            Some(serde_json::from_str(&*auth_str).unwrap())
        },
        None => None
    };

    let port: Option<u16> = match matched.value_of("port") {
        Some(port) => Some(port.parse().unwrap()),
        None => None
    };

    let workers: Option<u16> = match matched.value_of("workers") {
        Some(workers) => Some(workers.parse().unwrap()),
        None => None
    };

    database_check(&database, false);

    for db_read in &database_read {
        database_check(db_read, true);
    }

    hecate::start(
        database,
        database_read,
        port,
        workers,
        schema,
        auth
    );
}

fn database_check(conn_str: &String, is_read: bool) {
    match postgres::Connection::connect(format!("postgres://{}", conn_str), postgres::TlsMode::None) {
        Ok(conn) => {
            let conn_type = match is_read {
                true => String::from("READ"),
                false => String::from("READ/WRITE")
            };

            match conn.query("
                SELECT id FROM geo LIMIT 1
            ", &[]) {
                Ok(_) => (),
                Err(err) => {
                    println!("ERROR: Connection unable to {} geo table using {}", conn_type, conn_str);
                    println!("ERROR: {}", err.description());
                    println!("ERROR: Caused by: {}", err.cause().unwrap());
                    std::process::exit(1);
                }
            }
        },
        Err(err) => {
            println!("ERROR: Unable to connect to {}", conn_str);
            println!("ERROR: {}", err.description());
            println!("ERROR: caused by: {}", err.cause().unwrap());

            std::process::exit(1);
        }
    }
}
