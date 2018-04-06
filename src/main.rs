extern crate hecate;
#[macro_use] extern crate clap;
extern crate serde_json;
extern crate tempdir;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use clap::App;

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let matched = App::from_yaml(cli_cnf).get_matches();

    let database = matched.value_of("database").unwrap_or("postgres@localhost:5432/hecate");

    let schema: Option<serde_json::value::Value> = match matched.value_of("schema") {
        Some(schema_path) => {
            let mut schema_file = match File::open(&Path::new(schema_path)) {
                Ok(file) => file,
                Err(_) => panic!("Failed to open file at: {}", schema_path)
            };

            let mut schema_str = String::new();

            schema_file.read_to_string(&mut schema_str).unwrap();

            let schema_json: serde_json::value::Value = serde_json::from_str(&schema_str).unwrap();

            Some(schema_json)
        },
        None => None
    };

    hecate::start(String::from(database), schema);
}
