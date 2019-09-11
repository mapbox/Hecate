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
            assert_eq!(resp.text().unwrap(), "{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Unauthorized\"}");
            assert!(resp.status().is_client_error());
        }

        {
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/schema")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "{\"code\":404,\"reason\":\"No schema Validation Enforced\",\"status\":\"Not Found\"}");
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

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let mut resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert_eq!(resp.text().unwrap(), "{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Unauthorized\"}");
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
            assert_eq!(resp.text().unwrap(), "{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Unauthorized\"}");
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
            assert_eq!(resp.text().unwrap(), "{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Unauthorized\"}");
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

        {
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/auth").send().unwrap();

            let resp_value: Value = resp.json().unwrap();

            assert_eq!(resp_value, json!({
                "default": "public",
                "server": "public",
                "webhooks": {
                    "list": "user",
                    "update": "user",
                    "delete": "user"
                },
                "meta": {
                    "get": "user",
                    "list": "user",
                    "set": "user"
                },
                "schema": {
                    "get": "user"
                },
                "stats": {
                    "get": "user",
                    "bounds": "user"
                },
                "mvt": {
                    "get": "user",
                    "regen": "user",
                    "delete": "user",
                    "meta": "user"
                },
                "user": {
                    "info": "self",
                    "list": "user",
                    "create": "public",
                    "create_session": "self"
                },
                "style": {
                    "create": "self",
                    "patch": "self",
                    "set_public": "self",
                    "set_private": "self",
                    "delete": "self",
                    "get": "user",
                    "list": "user"
                },
                "delta": {
                    "get": "user",
                    "list": "user"
                },
                "feature": {
                    "force": "user",
                    "create": "user",
                    "get": "user",
                    "history": "user"
                },
                "bounds": {
                    "list": "user",
                    "create": "user",
                    "delete": "user",
                    "get": "user"
                },
                "osm": {
                    "get": "user",
                    "create": "user"
                },
                "clone": {
                    "get": "user",
                    "query": "user"
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
