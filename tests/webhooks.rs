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
    use serde_json;

    #[test]
    fn webhooks() {
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
            "--auth", env::current_dir().unwrap().join("tests/fixtures/auth.closed.json").to_str().unwrap()
        ]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { // Create User
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            conn.execute("
                INSERT INTO users (username, password, email)
                    VALUES ('ingalls', crypt('yeaheh', gen_salt('bf', 10)), 'ingalls@protonmail.com')
            ", &[]).unwrap();
        }

        {
            let client = reqwest::Client::new();

            let mut resp = client.get("http://localhost:8000/api/webhooks")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([]));

            assert!(resp.status().is_success());
        }

        {
            let client = reqwest::Client::new();

            let mut resp = client.post("http://localhost:8000/api/webhooks")
                .body(r#"{
                    "name": "webhook",
                    "url": "https://example.com",
                    "actions": ["delta"]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 1,
                "name": "webhook",
                "url": "https://example.com",
                "actions": ["delta"]
            }));
            assert!(resp.status().is_success());
        }

        {
            let client = reqwest::Client::new();

            let mut resp = client.post("http://localhost:8000/api/webhooks")
                .body(r#"{
                    "name": "webhook",
                    "url": "https://example.com",
                    "actions": ["delta", "fake"]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Invalid Action",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        {
            let client = reqwest::Client::new();

            let mut resp = client.get("http://localhost:8000/api/webhooks")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([{
                "id": 1,
                "name": "webhook",
                "url": "https://example.com",
                "actions": ["delta"]
            }]));

            assert!(resp.status().is_success());
        }

        {
            let client = reqwest::Client::new();

            let mut resp = client.post("http://localhost:8000/api/webhooks/1")
                .body(r#"{
                    "name": "webhook modify",
                    "url": "https://example.com/modify",
                    "actions": ["delta"]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 1,
                "name": "webhook modify",
                "url": "https://example.com/modify",
                "actions": ["delta"]
            }));
            assert!(resp.status().is_success());
        }


        {
            let client = reqwest::Client::new();

            let resp = client.delete("http://localhost:8000/api/webhooks/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
