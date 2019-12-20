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
    fn user_modify() {
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

        {
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();

            conn.execute("
                UPDATE users SET access = 'admin' WHERE id = 1;
            ", &[]).unwrap();
        }

        { //Create Second User
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=future_admin&password=yeahehyeah&email=fake@example.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { // An admin can get user info about an arbitrary user
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/2")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 2,
                "access": "default",
                "username": "future_admin",
                "email": "fake@example.com",
                "meta": {}
            }));
        }

        { // An admin can change user information
            let client = reqwest::Client::new();
            let resp = client.post("http://localhost:8000/api/user/2")
                .body(r#"{
                    "access": "default",
                    "username": "changed",
                    "email": "changed@example.com",
                    "meta": {
                        "random": "key"
                    }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        { // Ensure information was changed about the given user
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/2")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 2,
                "access": "default",
                "username": "changed",
                "email": "changed@example.com",
                "meta": {
                    "random": "key"
                }
            }));
        }

        let cookie: String;

        // Create a session which should be destroyted in a subsequent call when
        // the user account is disabled
        {
            let client = reqwest::Client::new();
            let mut session_resp = client.get("http://localhost:8000/api/user/session")
                .basic_auth("changed", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(session_resp.status().is_success());
            assert_eq!(session_resp.text().unwrap(), "true");

            let cookies: Vec<reqwest::cookie::Cookie> = session_resp.cookies().into_iter().collect();

            assert_eq!(cookies[0].name(), "session");
            assert!(cookies[0].value().len() > 0);

            cookie = format!("{}={}", cookies[0].name(), cookies[0].value());
        }

        { // Access the style create (FULL scope) endpoint with cookie
            let client = reqwest::Client::new();
            let mut create_style_resp = client.post("http://localhost:8000/api/style")
                .body(r#"{
                    "name": "Awesome Style",
                    "style": "I am a style"
                }"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();

            assert_eq!(create_style_resp.text().unwrap(), "1");
            assert!(create_style_resp.status().is_success());
        }

        { // Disable user account
            let client = reqwest::Client::new();
            let resp = client.post("http://localhost:8000/api/user/2")
                .body(r#"{
                    "access": "disabled",
                    "username": "changed",
                    "email": "changed@example.com",
                    "meta": {
                        "random": "key"
                    }
                }"#)
                .basic_auth("ingalls", Some("yeahehyeah"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        { // Ensure information was changed about the given user
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/2")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 2,
                "access": "disabled",
                "username": "changed",
                "email": "changed@example.com",
                "meta": {
                    "random": "key"
                }
            }));
        }

        // Access the style create (FULL scope) endpoint with cookie - this should fail
        // as when a user account is disabled, all sessions/tokens are destroyed
        {
            let client = reqwest::Client::new();
            let create_style_resp = client.post("http://localhost:8000/api/style")
                .body(r#"{
                    "name": "Awesome Style",
                    "style": "I am a style"
                }"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();

            assert!(create_style_resp.status().is_client_error());
        }

        // The account should also not be able to make any subsequent API calls as it has been
        // disabled
        {
            let client = reqwest::Client::new();
            let session_resp = client.get("http://localhost:8000/api/user/session")
                .basic_auth("changed", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(session_resp.status().is_client_error());
        }

        // The password for a disabled user account is changed automatically
        //
        // Change the password back to something we know and make sure the endpoint still fails
        // as it has access: disabled
        {
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();

            conn.execute("
                UPDATE users
                    SET
                        password = crypt('yeahehyeah', gen_salt('bf', 10))
                    WHERE id = 2;
            ", &[]).unwrap();

            let client = reqwest::Client::new();
            let session_resp = client.get("http://localhost:8000/api/user/session")
                .basic_auth("changed", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(session_resp.status().is_client_error());
        }

        server.kill().unwrap();
    }
}
