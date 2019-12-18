extern crate reqwest;
extern crate postgres;
#[macro_use] extern crate serde_json;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::env;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use std::process::Command;
    use std::time::Duration;
    use std::thread;
    use reqwest;
    use serde_json::value::Value;

    #[test]
    fn auth_disabled() {
        {
            let conn = Connection::connect("postgres://postgres@localhost:5432", TlsMode::None).unwrap();

            conn.execute("
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE
                    pg_stat_activity.datname = 'hecate'
                    AND pid <> pg_backend_pid();
            ", &[]).unwrap();

            conn.execute("
                DROP DATABASE IF EXISTS hecate;
            ", &[]).unwrap();

            conn.execute("
                CREATE DATABASE hecate;
            ", &[]).unwrap();

            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();

            let mut file = File::open("./src/schema.sql").unwrap();
            let mut table_sql = String::new();
            file.read_to_string(&mut table_sql).unwrap();
            conn.batch_execute(&*table_sql).unwrap();
        }

        let mut server = Command::new("cargo").args(&[
            "run",
            "--",
            "--auth", env::current_dir().unwrap().join("tests/fixtures/auth.disabled.json").to_str().unwrap()
        ]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        {
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/auth").send().unwrap();

            let resp_value: Value = resp.json().unwrap();

            assert_eq!(resp_value, json!({
                "default": "public",
                "server": "disabled",
                "webhooks": {
                    "get": "disabled",
                    "set": "disabled"
                },
                "meta": {
                    "get": "disabled",
                    "set": "disabled"
                },
                "schema": {
                    "get": "disabled"
                },
                "stats": {
                    "get": "disabled"
                },
                "mvt": {
                    "get": "disabled",
                    "regen": "disabled",
                    "delete": "disabled",
                    "meta": "disabled"
                },
                "user": {
                    "info": "disabled",
                    "list": "disabled",
                    "create": "disabled",
                    "create_session": "disabled"
                },
                "style": {
                    "create": "disabled",
                    "patch": "disabled",
                    "set_public": "disabled",
                    "set_private": "disabled",
                    "delete": "disabled",
                    "get": "disabled",
                    "list": "disabled"
                },
                "delta": {
                    "get": "disabled",
                    "list": "disabled"
                },
                "feature": {
                    "force": "disabled",
                    "create": "disabled",
                    "get": "disabled",
                    "history": "disabled"
                },
                "bounds": {
                    "list": "disabled",
                    "create": "disabled",
                    "delete": "disabled",
                    "get": "disabled"
                },
                "osm": {
                    "get": "disabled",
                    "create": "disabled"
                },
                "clone": {
                    "get": "disabled",
                    "query": "disabled"
                },
                "auth": {
                    "get": "public"
                }
            }));
            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
