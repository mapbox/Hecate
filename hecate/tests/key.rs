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
    fn key() {
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
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeahehyeah&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Key Value Must Be A String
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "key": 1243,
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "{\"feature\":{\"action\":\"create\",\"geometry\":{\"coordinates\":[0.0,0.0],\"type\":\"Point\"},\"key\":1243,\"message\":\"Creating a Point\",\"properties\":{\"number\":\"123\"},\"type\":\"Feature\"},\"id\":null,\"message\":\"key must be a string value\"}");
        }

        { //Create Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
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
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();

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

        { //Check Point 1 - Key API
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature?key=Q1233").unwrap();

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

        { //Create Point 2
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "key": "Rando",
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

        { //Check Point 2
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/2").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 2,
                "type": "Feature",
                "key": "Rando",
                "version": 1,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 0.0, 0.0 ] }
            }));

            assert!(resp.status().is_success());
        }

        { //Check Point 2 - Key API
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature?key=Rando").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 2,
                "type": "Feature",
                "key": "Rando",
                "version": 1,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
            }));

            assert!(resp.status().is_success());
        }

        { //Create Point 3 - explicit NULL Key
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "key": null,
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

        { //Check Point 3
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/3").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 3,
                "type": "Feature",
                "key": null,
                "version": 1,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 0.0, 0.0 ] }
            }));

            assert!(resp.status().is_success());
        }

        { //Create Point 4 - undefined Key
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
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

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Check Point 4
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/4").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 4,
                "type": "Feature",
                "key": null,
                "version": 1,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 0.0, 0.0 ] }
            }));

            assert!(resp.status().is_success());
        }

        { //Create Point 5 - explicit NULL Key (should pass, duplicate NULLs are allowed)
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "key": null,
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Check Point 5
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/5").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 5,
                "type": "Feature",
                "key": null,
                "version": 1,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 0.0, 0.0 ] }
            }));

            assert!(resp.status().is_success());
        }


        { //Create Duplicate Key Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
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

            assert_eq!(resp.text().unwrap(), "{\"feature\":{\"action\":\"create\",\"geometry\":{\"coordinates\":[0.0,0.0],\"type\":\"Point\"},\"key\":\"Q1233\",\"message\":\"Creating a Point\",\"properties\":{\"number\":\"123\"},\"type\":\"Feature\"},\"id\":null,\"message\":\"Duplicate Key Value\"}");
            assert!(resp.status().is_client_error());
        }

        { //Modify Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "key": "12-34",
                    "type": "Feature",
                    "version": 1,
                    "action": "modify",
                    "message": "Modify a point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Check Point 1
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 1,
                "type": "Feature",
                "key": "12-34",
                "version": 2,
                "properties": { "number": "123" },
                "geometry": { "type": "Point", "coordinates": [ 1.0, 1.0 ] }
            }));

            assert!(resp.status().is_success());
        }

        { //Modify Point - Duplicate Key Error
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 2,
                    "key": "12-34",
                    "type": "Feature",
                    "version": 1,
                    "action": "modify",
                    "message": "Modify a point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "{\"feature\":{\"action\":\"modify\",\"geometry\":{\"coordinates\":[1.0,1.0],\"type\":\"Point\"},\"id\":2,\"key\":\"12-34\",\"message\":\"Modify a point\",\"properties\":{\"number\":\"123\"},\"type\":\"Feature\",\"version\":1},\"id\":2,\"message\":\"Duplicate Key Value\"}");
            assert!(resp.status().is_client_error());
        }

        { //Delete Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 2,
                    "action": "delete",
                    "message": "Delete a point",
                    "properties": null,
                    "geometry": null
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert!(resp.status().is_client_error());
        }


        { //Restore Point - Duplicate Key Error
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "key": "Rando",
                    "type": "Feature",
                    "version": 3,
                    "action": "restore",
                    "message": "Restore previously deleted point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "{\"feature\":{\"action\":\"restore\",\"geometry\":{\"coordinates\":[1.0,1.0],\"type\":\"Point\"},\"id\":1,\"key\":\"Rando\",\"message\":\"Restore previously deleted point\",\"properties\":{\"number\":\"123\"},\"type\":\"Feature\",\"version\":3},\"id\":1,\"message\":\"Duplicate Key Value\"}");
            assert!(resp.status().is_client_error());
        }

        server.kill().unwrap();
    }
}
