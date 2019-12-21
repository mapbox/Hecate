extern crate hecate;
#[macro_use] extern crate clap;
extern crate serde_json;
extern crate postgres;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use hecate::auth::CustomAuth;
use hecate::auth::AuthModule;
use std::error::Error;
use clap::App;

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let matched = App::from_yaml(cli_cnf).get_matches();

    let database = String::from(matched.value_of("database").unwrap_or("hecate@localhost:5432/hecate"));

    let database_sandbox = match matched.values_of("database_sandbox") {
        None => vec![String::from("hecate_read@localhost:5432/hecate")],
        Some(db_read) => db_read.map(|db| String::from(db)).collect()
    };

    let database_replica = match matched.values_of("database_replica") {
        None => vec![database.clone()],
        Some(db_read) => db_read.map(|db| String::from(db)).collect()
    };

    let schema: Option<serde_json::Value> = match matched.value_of("schema") {
        Some(schema_path) => {
            let mut schema_file = match File::open(&Path::new(schema_path)) {
                Ok(file) => file,
                Err(_) => panic!("Failed to open schema file at: {}", schema_path)
            };

            let mut schema_str = String::new();

            schema_file.read_to_string(&mut schema_str).unwrap();

            let schema_json: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

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

            let mut auth = String::new();

            match auth_file.read_to_string(&mut auth) {
                Err(err) => panic!("Could not read auth file: {}", err.to_string()),
                _ => ()
            };

            let auth: serde_json::Value = match serde_json::from_str(&*auth) {
                Ok(auth) => auth,
                Err(err) => panic!("Auth file is not valid JSON: {}", err.to_string())
            };

            let auth = match CustomAuth::parse(Some(&auth)) {
                Ok(auth) => auth,
                Err(err) => panic!("{}", err.as_log())
            };

            Some(*auth)
        },
        None => None
    };

    let port: Option<u16> = match matched.value_of("port") {
        Some(port) => match port.parse() {
            Ok(port) => Some(port),
            _ => { panic!("Port must be an integer > 1000") }
        },
        None => None
    };

    let workers: Option<u16> = match matched.value_of("workers") {
        Some(workers) => match workers.parse() {
            Ok(workers) => Some(workers),
            _ => { panic!("workers arg must be integer value") }
        },
        None => None
    };

    database_check(&database, false);

    for db_replica in &database_replica {
        database_check(db_replica, true);
    }

    for db_sandbox in &database_sandbox {
        database_check(db_sandbox, true);
    }

    hecate::start(
        hecate::db::Database::new(database, database_replica, database_sandbox),
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

            if !is_read {
                match conn.query("
                    SELECT
                        (regexp_matches(version(), 'PostgreSQL (.*?) '))[1]::FLOAT AS postgres_v,
                        (regexp_matches(postgis_version(), '^(.*?) '))[1]::FLOAT AS postgis_v
                ", &[]) {
                    Ok(res) => {
                        if res.len() != 1 {
                            println!("ERROR: Connection unable obtain postgres version using ({}) {}", conn_type, conn_str);
                            std::process::exit(1);
                        }

                        let postgres_v: f64 = res.get(0).get(0);
                        if postgres_v < hecate::POSTGRES {
                            println!("ERROR: Hecate requires a min postgres version of {}", hecate::POSTGRES);
                            std::process::exit(1);
                        }

                        let postgis_v: f64 = res.get(0).get(1);
                        if postgis_v < hecate::POSTGIS {
                            println!("ERROR: Hecate requires a min postgis version of {}", hecate::POSTGIS);
                            std::process::exit(1);
                        }
                    },
                    Err(err) => {
                        println!("ERROR: Connection unable obtain postgres version using ({}) {}", conn_type, conn_str);
                        println!("ERROR: {}", err.description());
                        println!("ERROR: Caused by: {}", err.source().unwrap());
                        std::process::exit(1);
                    }
                }
            }

            match conn.query("
                SELECT id FROM geo LIMIT 1
            ", &[]) {
                Ok(_) => (),
                Err(err) => {
                    println!("ERROR: Connection unable to {} geo table using {}", conn_type, conn_str);
                    println!("ERROR: {}", err.description());
                    println!("ERROR: Caused by: {}", err.source().unwrap());
                    std::process::exit(1);
                }
            }
        },
        Err(err) => {
            println!("ERROR: Unable to connect to {}", conn_str);
            println!("ERROR: {}", err.description());
            println!("ERROR: caused by: {}", err.source().unwrap());

            std::process::exit(1);
        }
    }
}
