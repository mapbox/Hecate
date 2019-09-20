extern crate reqwest;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use std::process::Command;
    use std::time::Duration;
    use std::thread;
    use reqwest;

    #[test]
    fn history() {
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

        let mut server = Command::new("cargo").args(&[ "run" ]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { // Create User
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            conn.execute("
                INSERT INTO users (username, password, email)
                    VALUES ('ingalls', crypt('yeaheh', gen_salt('bf', 10)), 'ingalls@protonmail.com')
            ", &[]).unwrap();
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
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/1/history").unwrap();
            assert_eq!(resp.text().unwrap(), r#"[{"feat":{"action":"create","geometry":{"coordinates":[0.0,0.0],"type":"Point"},"id":1,"message":"Creating a Point","properties":{"number":"123"},"type":"Feature"},"id":1,"uid":1,"username":"ingalls"}]"#);
            assert!(resp.status().is_success());
        }

        { //Modify Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 1,
                    "action": "modify",
                    "message": "Modify a Point",
                    "properties": { "number": "123", "test": true },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/1/history").unwrap();
            assert_eq!(resp.text().unwrap(), r#"[{"feat":{"action":"modify","geometry":{"coordinates":[1.0,1.0],"type":"Point"},"id":1,"message":"Modify a Point","properties":{"number":"123","test":true},"type":"Feature","version":1},"id":2,"uid":1,"username":"ingalls"},{"feat":{"action":"create","geometry":{"coordinates":[0.0,0.0],"type":"Point"},"id":1,"message":"Creating a Point","properties":{"number":"123"},"type":"Feature"},"id":1,"uid":1,"username":"ingalls"}]"#);
            assert!(resp.status().is_success());
        }

        { //Delete Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 2,
                    "action": "delete",
                    "message": "Delete a Point",
                    "properties": null,
                    "geometry": null
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/1/history").unwrap();
            assert_eq!(resp.text().unwrap(), r#"[{"feat":{"action":"delete","geometry":null,"id":1,"message":"Delete a Point","properties":{},"type":"Feature","version":2},"id":3,"uid":1,"username":"ingalls"},{"feat":{"action":"modify","geometry":{"coordinates":[1.0,1.0],"type":"Point"},"id":1,"message":"Modify a Point","properties":{"number":"123","test":true},"type":"Feature","version":1},"id":2,"uid":1,"username":"ingalls"},{"feat":{"action":"create","geometry":{"coordinates":[0.0,0.0],"type":"Point"},"id":1,"message":"Creating a Point","properties":{"number":"123"},"type":"Feature"},"id":1,"uid":1,"username":"ingalls"}]"#);
            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
