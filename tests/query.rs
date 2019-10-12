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
    fn query() {
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
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Create a Point",
                    "properties": { "name": "I am Feature 1" },
                    "geometry": { "type": "Point", "coordinates": [ -77.01210021972656,38.925763232374514 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Create Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Create a Point",
                    "properties": { "name": "I am Feature 2" },
                    "geometry": { "type": "Point", "coordinates": [ -77.01210021972656,38.925763232374514 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Create Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Create a Point",
                    "properties": { "name": "I am Feature 3" },
                    "geometry": { "type": "Point", "coordinates": [ -77.01210021972656,38.925763232374514 ] }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Get Query - SELECT count(*) FROM geo
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/data/query?query=SELECT%20count(*)%20FROM%20geo")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            let mut body_str = String::from(resp.text().unwrap());
            body_str.pop();
            body_str.pop();
            assert_eq!(&*body_str, "{\"count\":3}");
            assert!(resp.status().is_success());
        }

        { //Get Query - SELECT props FROM geo &limit=1
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/data/query?limit=1&query=SELECT%20props%20FROM%20geo")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            let mut body_str = String::from(resp.text().unwrap());
            body_str.pop();
            body_str.pop();
            assert_eq!(&*body_str, "{\"props\":{\"name\": \"I am Feature 1\"}}");
            assert!(resp.status().is_success());
        }
        
        { //Get Query - Failed Insert
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8000/api/data/query?limit=1&query=INSERT%20INTO%20geo(props)%20VALUES%20(%27%7B%7D%27%3A%3ATEXT%3A%3AJSON)")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_server_error());
        }

        { //Get Query - Failed Delete
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8000/api/data/query?limit=1&query=DELETE%20FROM%20geo")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_server_error());
        }

        server.kill().unwrap();
    }
}
