extern crate reqwest;
extern crate postgres;

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

    #[test]
    fn auth_closed() {
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

        { //Create Username
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/schema").unwrap();
            assert_eq!(resp.text().unwrap(), "Not Authorized!");
            assert!(resp.status().is_client_error());
        }

        {
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/schema")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "No Schema Validation Enforced");
        }

        { //Create Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert_eq!(resp.text().unwrap(), "Not Authorized!");
            assert!(resp.status().is_client_error());
        }

        {
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8000/api/data/feature/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();
            assert_eq!(resp.text().unwrap(), "Not Authorized!");
            assert!(resp.status().is_client_error());
        }

        {
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8000/api/deltas")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/delta/1").unwrap();
            assert_eq!(resp.text().unwrap(), "Not Authorized!");
            assert!(resp.status().is_client_error());
        }

        {
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8000/api/delta/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
