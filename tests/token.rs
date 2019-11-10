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

    #[test]
    fn token() {
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
            "--auth", env::current_dir().unwrap().join("tests/fixtures/auth.default.json").to_str().unwrap()
        ]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { // Create User
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            conn.execute("
                INSERT INTO users (username, password, email)
                    VALUES ('ingalls', crypt('yeahehyeah', gen_salt('bf', 10)), 'ingalls@protonmail.com')
            ", &[]).unwrap();
        }

        { // Attempt to access default server
            let mut resp = reqwest::get("http://localhost:8000/").unwrap();
            assert_eq!(resp.status().as_u16(), 200);
            assert_eq!(resp.text().unwrap(), "Hello World!");
        }

        let token: String;

        { // Create Token
            let client = reqwest::Client::new();

            let mut resp = client.post("http://localhost:8000/api/user/token")
                .body(r#"{
                    "name": "JOSM Token",
                    "hours": 5
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body["name"], json!("Access Token"));
            assert_eq!(json_body["uid"], json!(1));

            assert!(resp.status().is_success());

            token = json_body["token"].as_str().unwrap().to_string();
        }

        { // Access the capabilities (READ scope) endpoint without token
            let resp = reqwest::get("http://localhost:8000/api/capabilities").unwrap();
            assert_eq!(resp.status().as_u16(), 401);
        }

        { // Access the capabilities (READ scope) endpoint with token
            let resp = reqwest::get(format!("http://localhost:8000/token/{}/api/capabilities", token).as_str()).unwrap();
            assert_eq!(resp.status().as_u16(), 200);
        }

        { // Access the feature create (FULL scope) endpoint without token
            let client = reqwest::Client::new();
            let resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.status().as_u16(), 401);
        }

        { // Access the feature create (FULL scope) endpoint with token
            let client = reqwest::Client::new();
            let resp = client.post(format!("http://localhost:8000/token/{}/api/data/feature", token).as_str())
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.status().as_u16(), 401);
        }

        { // Access the feature create (FULL scope) endpoint with token and basic auth
            let client = reqwest::Client::new();
            let resp = client.post(format!("http://localhost:8000/token/{}/api/data/feature", token).as_str())
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.status().as_u16(), 200);
        }

        { // Delete Token
            let client = reqwest::Client::new();

            let resp = client.delete(format!("http://localhost:8000/api/user/token/{}", token).as_str())
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert_eq!(resp.status().as_u16(), 200);
        }

        { // Access the capabilities endpoint with token
            let resp = reqwest::get(format!("http://localhost:8000/token/{}/api/capabilities", token).as_str()).unwrap();
            assert_eq!(resp.status().as_u16(), 401);
        }

        server.kill().unwrap();
    }
}
