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
    use hecate::webhooks::WebHook;

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

        { // Create Username (ingalls)
            let mut resp = reqwest::get("http://0.0.0.0:8000/api/user/create?username=ingalls&password=yeahehyeah&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        {   // Make an empty get request
            let client = reqwest::Client::new();

            let mut resp = client.get("http://0.0.0.0:8000/api/webhooks")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([]));

            assert!(resp.status().is_success());
        }

        { // Create a new webhook and generate a secret
            let client = reqwest::Client::new();

            let mut resp = client.post("http://0.0.0.0:8000/api/webhooks")
                .body(r#"{
                    "name": "webhook",
                    "url": "https://example.com",
                    "actions": ["delta"]
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();
            let webhook: WebHook = serde_json::from_value(json_body).unwrap();
            assert_eq!(webhook.id, Some(1));
            assert_eq!(webhook.name, String::from("webhook"));
            assert_eq!(webhook.url, String::from("https://example.com"));
            assert_eq!(webhook.actions, vec![String::from("delta")]);
            assert!(webhook.secret.is_some());
            assert_eq!(webhook.secret.unwrap().len(), 30);
        }

        { // Create a new webhook, passing in a custom secret
            let client = reqwest::Client::new();

            let mut resp = client.post("http://0.0.0.0:8000/api/webhooks")
                .body(r#"{
                    "name": "webhook",
                    "url": "https://example.com",
                    "actions": ["delta"],
                    "secret": "my secret"
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 2,
                "name": "webhook",
                "url": "https://example.com",
                "actions": ["delta"],
                "secret": "my secret"
            }));
            assert!(resp.status().is_success());
        }

        { // Err, invalid action passed to create
            let client = reqwest::Client::new();

            let mut resp = client.post("http://0.0.0.0:8000/api/webhooks")
                .body(r#"{
                    "name": "webhook",
                    "url": "https://example.com",
                    "actions": ["delta", "fake"]
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
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

        { // Err, invalid url passed to create
            let client = reqwest::Client::new();

            let mut resp = client.post("http://0.0.0.0:8000/api/webhooks")
                .body(r#"{
                    "name": "webhook",
                    "url": "bad-url",
                    "actions": ["delta", "fake"]
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
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

        { // List webhooks
            let client = reqwest::Client::new();

            let mut resp = client.get("http://0.0.0.0:8000/api/webhooks")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([{
                "id": 1,
                "name": "webhook",
                "url": "https://example.com",
                "actions": ["delta"]
            }, {
                "id": 2,
                "name": "webhook",
                "url": "https://example.com",
                "actions": ["delta"]
            }]));

            assert!(resp.status().is_success());
        }

        { // Get webhook
            let client = reqwest::Client::new();

            let mut resp = client.get("http://0.0.0.0:8000/api/webhooks/1")
                .basic_auth("ingalls", Some("yeahehyeah"))
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

        { // Err, invalid id passed to get webhook
            let client = reqwest::Client::new();

            let mut resp = client.get("http://0.0.0.0:8000/api/webhooks/10")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();
            assert_eq!(json_body, json!({
                "code": 404,
                "reason": "Webhook Not Found",
                "status": "Not Found"
            }));
            assert!(resp.status().is_client_error());
        }

        { // Update webhooks
            let client = reqwest::Client::new();

            let mut resp = client.post("http://0.0.0.0:8000/api/webhooks/1")
                .body(r#"{
                    "name": "webhook update",
                    "url": "https://example.com/modify",
                    "actions": ["delta"]
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 1,
                "name": "webhook update",
                "url": "https://example.com/modify",
                "actions": ["delta"]
            }));
            assert!(resp.status().is_success());
        }

        { // Err, invalid action passed to update
            let client = reqwest::Client::new();

            let mut resp = client.post("http://0.0.0.0:8000/api/webhooks/1")
                .body(r#"{
                    "name": "webhook update",
                    "url": "https://example.com/modify",
                    "actions": ["fake"]
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
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

        { // Err, invalid url passed to update
            let client = reqwest::Client::new();

            let mut resp = client.post("http://0.0.0.0:8000/api/webhooks/1")
                .body(r#"{
                    "name": "webhook update",
                    "url": "bad-url",
                    "actions": ["delta"]
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "code": 422,
                "reason": "Invalid webhook url",
                "status": "Unprocessable Entity"
            }));
            assert!(resp.status().is_client_error());
        }

        {
            let client = reqwest::Client::new();

            let resp = client.delete("http://0.0.0.0:8000/api/webhooks/1")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
