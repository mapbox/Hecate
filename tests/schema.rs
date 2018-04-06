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
    fn schema() {
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
            "--schema", env::current_dir().unwrap().join("tests/fixtures/source_schema.json").to_str().unwrap()
        ]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { //Create Username
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Get Schema in use
            let mut resp = reqwest::get("http://localhost:8000/api/schema").unwrap();
            assert_eq!(resp.text().unwrap(), "{\"$schema\":\"http://json-schema.org/draft-04/schema#\",\"description\":\"Validate addresses source\",\"properties\":{\"number\":{\"description\":\"Number of the building.\",\"type\":\"string\"},\"source\":{\"description\":\"Name of the source where the data comes from\",\"type\":\"string\"},\"street\":{\"description\":\"Name Array of street names to which this address belongs\",\"items\":{\"propeties\":{\"display\":{\"description\":\"Single name string of a potential road name\",\"type\":\"string\"},\"priority\":{\"description\":\"Used to determine the primary name of a feature\",\"type\":\"integer\"}},\"required\":[\"display\"],\"type\":\"object\"},\"type\":\"array\"}},\"required\":[\"source\",\"number\",\"street\"],\"title\":\"Address source\",\"type\":\"object\"}");
            assert!(resp.status().is_success());
        }

        { //Create Point Failing Schema Validation
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

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Feature properties do not pass schema definition");
        }

        { //Create Point Almost Passing Schema Validation
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": {
                        "source": "Test Data",
                        "number": "123",
                        "street": [{ "test": "123" }]
                    },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Feature properties do not pass schema definition");
        }

        { //Create Point Passing Schema Validation
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": {
                        "source": "Test Data",
                        "number": "123",
                        "street": [{ "display": "Main Street" }]
                    },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        server.kill().unwrap();
    }
}
