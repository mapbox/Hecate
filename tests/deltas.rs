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
    fn deltas() {
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

        { //Create Points
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/features")
                .body(r#"{
                    "type": "FeatureCollection",
                    "message": "Basic Creation",
                    "features": [{
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1,1 ]
                        }
                    }, {
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1.1,1.1 ]
                        }
                    }, {
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1.2,1.2 ]
                        }
                    }]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Modify Points
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/features")
                .body(r#"{
                    "type": "FeatureCollection",
                    "message": "Basic Modify",
                    "features": [{
                        "id": 1,
                        "type": "Feature",
                        "version": 1,
                        "action": "modify",
                        "properties": {
                            "shop": false
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 2,2 ]
                        }
                    }, {
                        "id": 2,
                        "type": "Feature",
                        "version": 1,
                        "action": "modify",
                        "properties": {
                            "shop": true,
                            "building": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 0.1, 0.1 ]
                        }
                    }, {
                        "id": 3,
                        "type": "Feature",
                        "version": 1,
                        "action": "modify",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 2.2, 2.2 ]
                        }
                    }]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create More Points
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/features")
                .body(r#"{
                    "type": "FeatureCollection",
                    "message": "Basic Creation",
                    "features": [{
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1,1 ]
                        }
                    }, {
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1.1,1.1 ]
                        }
                    }, {
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1.2,1.2 ]
                        }
                    }]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/delta/1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body["affected"], json!([1, 2, 3]));
            assert_eq!(json_body["id"], json!(1));
            assert_eq!(json_body["props"], json!({ "message": "Basic Creation" }));
            assert_eq!(json_body["uid"], json!(1));
            assert_eq!(json_body["username"], json!("ingalls"));

            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/delta/2").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body["affected"], json!([1, 2, 3]));
            assert_eq!(json_body["id"], json!(2));
            assert_eq!(json_body["props"], json!({ "message": "Basic Modify" }));
            assert_eq!(json_body["uid"], json!(1));
            assert_eq!(json_body["username"], json!("ingalls"));

            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 3);

            assert_eq!(json_body[0]["id"], 3);
            assert_eq!(json_body[1]["id"], 2);
            assert_eq!(json_body[2]["id"], 1);

            assert!(resp.status().is_success());
        }

        { //Test limit param
            let mut resp = reqwest::get("http://localhost:8000/api/deltas?limit=1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 1);

            assert_eq!(json_body[0]["id"], 3);

            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/deltas?offset=2").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 1);
            assert_eq!(json_body[0]["id"], json!(1));

            assert!(resp.status().is_success());
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/deltas?offset=2&limit=1").unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 1);
            assert_eq!(json_body[0]["id"], json!(1));

            assert!(resp.status().is_success());
        }

        { //Test Start Value
            let mut resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            let start = String::from(json_body[0]["created"].as_str().unwrap());

            let mut resp = reqwest::get(&*format!("http://localhost:8000/api/deltas?start={}", &start)).unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 2);
            assert_eq!(json_body[0]["id"], json!(2));
            assert_eq!(json_body[1]["id"], json!(1));

            assert!(resp.status().is_success());
        }

        { //Test Start And Limit
            let mut resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            let start = String::from(json_body[0]["created"].as_str().unwrap());

            let mut resp = reqwest::get(&*format!("http://localhost:8000/api/deltas?start={}&limit=1", &start)).unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 1);
            assert_eq!(json_body[0]["id"], json!(2));

            assert!(resp.status().is_success());
        }

        { //Test End Value
            let mut resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            let end = String::from(json_body[2]["created"].as_str().unwrap());

            let mut resp = reqwest::get(&*format!("http://localhost:8000/api/deltas?end={}", &end)).unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 2);
            assert_eq!(json_body[0]["id"], json!(3));
            assert_eq!(json_body[1]["id"], json!(2));

            assert!(resp.status().is_success());
        }

        { //Test End Value & Limit
            let mut resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            let end = String::from(json_body[2]["created"].as_str().unwrap());

            let mut resp = reqwest::get(&*format!("http://localhost:8000/api/deltas?end={}&limit=1", &end)).unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 1);
            assert_eq!(json_body[0]["id"], json!(3));

            assert!(resp.status().is_success());
        }

        { //Test Start & End
            let mut resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            let start = String::from(json_body[0]["created"].as_str().unwrap());
            let end = String::from(json_body[2]["created"].as_str().unwrap());

            println!("http://localhost:8000/api/deltas?start={}&end={}", &start, &end);
            let mut resp = reqwest::get(&*format!("http://localhost:8000/api/deltas?start={}&end={}", &start, &end)).unwrap();
            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body.as_array().unwrap().len(), 1);
            assert_eq!(json_body[0]["id"], json!(2));

            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
