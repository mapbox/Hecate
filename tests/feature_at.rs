extern crate reqwest;
extern crate postgres;
#[macro_use] extern crate serde_json;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use std::process::Command;
    use std::time::Duration;
    use std::thread;
    use reqwest;
    use serde_json;

    #[test]
    fn feature_at() {
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

        { //Create Username
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/user/create?username=ingalls&password=yeahehyeah&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://0.0.0.0:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "key": "Q1233",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Check Point 1
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature/1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 1,
                "type": "Feature",
                "key": "Q1233",
                "version": 1,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 0.0, 0.0 ] }
            }));

            assert!(resp.status().is_success());
        }

        { //Invalid Point Lng,Lat
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Point must be Lng,Lat",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { //Invalid Point Lng,Lat - non numeric lng
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=hey%2C1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Longitude coordinate must be numeric",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { //Invalid Point Lng,Lat - non numeric lat
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=1%2Chey").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Latitude coordinate must be numeric",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { //Invalid Point Lng,Lat - Lng out of bounds neg
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=-190%2C1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Longitude exceeds bounds",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { //Invalid Point Lng,Lat - Lng out of bounds pos
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=190%2C1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Longitude exceeds bounds",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { //Invalid Point Lng,Lat - Lat out of bounds neg
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=1%2C-100").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Latitude exceeds bounds",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { //Invalid Point Lng,Lat - Lat out of bounds pos
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=1%2C100").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "Latitude exceeds bounds",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { //Check Point 1
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/feature?point=0.0%2C0.0").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 1,
                "type": "Feature",
                "key": "Q1233",
                "version": 1,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
            }));

            assert!(resp.status().is_success());
        }

        { // Check bbox - minX in bbox out of range
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/features?bbox=-181.0,-30.600094,56.162109,46.377254").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "BBOX minX value must be a number between -180 and 180",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { // Check bbox - minY in bbox out of range
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/features?bbox=-107.578125,-100.600094,56.162109,46.377254").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "BBOX minY value must be a number between -90 and 90",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { // Check bbox - maxX in bbox out of range
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/features?bbox=-107.578125,-30.600094,190.162109,46.377254").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "BBOX maxX value must be a number between -180 and 180",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { // Check bbox - maxY in bbox out of range
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/features?bbox=-107.578125,-30.600094,56.162109,100.377254").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "BBOX maxY value must be a number between -90 and 90",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { // Check bbox - minX > maxX 
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/features?bbox=107.578125,-30.600094,56.162109,46.377254").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "BBOX minX value cannot be greater than maxX value",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }

        { // Check bbox - minY > maxY
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/data/features?bbox=-107.578125,30.600094,56.162109,-46.377254").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 400,
                "reason": "BBOX minY value cannot be greater than maxY value",
                "status": "Bad Request"
            }));
            assert!(resp.status().is_client_error());
        }


        server.kill().unwrap();
    }
}
