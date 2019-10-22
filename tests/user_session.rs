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
    fn user_session() {
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

        { // Create Username
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeahehyeah&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        {
            let client = reqwest::Client::new();
            let cookie: String;
            let token: String;

            { // Create a new session given username & password
                let mut session_resp = client.get("http://localhost:8000/api/user/session")
                    .basic_auth("ingalls", Some("yeahehyeah"))
                    .send()
                    .unwrap();

                assert!(session_resp.status().is_success());
                assert_eq!(session_resp.text().unwrap(), "true");

                let cookies: Vec<reqwest::cookie::Cookie> = session_resp.cookies().into_iter().collect();

                assert_eq!(cookies[0].name(), "session");
                assert!(cookies[0].value().len() > 0);

                cookie = format!("{}={}", cookies[0].name(), cookies[0].value());
            }

            { // Create Token
                let mut token_resp = client.post("http://localhost:8000/api/user/token")
                    .body(r#"{
                        "name": "JOSM Token",
                        "hours": 5
                    }"#)
                    .basic_auth("ingalls", Some("yeahehyeah"))
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .send()
                    .unwrap();

                let json_body: serde_json::value::Value = token_resp.json().unwrap();
                assert_eq!(json_body["name"], json!("Access Token"));
                assert_eq!(json_body["uid"], json!(1));
                assert!(token_resp.status().is_success());
                token = json_body["token"].as_str().unwrap().to_string();
            }

            { // Access the style create (FULL scope) endpoint with cookie
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

            { // Access the styles get (READ scope) endpoint with cookie
                let mut get_style_resp = client.get("http://localhost:8000/api/style/1")
                    .header(reqwest::header::COOKIE, cookie.clone())
                    .send()
                    .unwrap();

                assert_eq!(get_style_resp.text().unwrap(), r#"{"id":1,"name":"Awesome Style","public":false,"style":"I am a style","uid":1,"username":"ingalls"}"#);
                assert!(get_style_resp.status().is_success());
            }

            { // Access the styles get (READ scope) endpoint with cookie and token
                let mut get_style_resp = client.get(format!("http://localhost:8000/token/{}/api/style/1", token).as_str())
                    .header(reqwest::header::COOKIE, cookie.clone())
                    .send()
                    .unwrap();
                assert_eq!(get_style_resp.text().unwrap(), r#"{"id":1,"name":"Awesome Style","public":false,"style":"I am a style","uid":1,"username":"ingalls"}"#);
                assert!(get_style_resp.status().is_success());
            }

            { // Access the style delete (FULL scope) endpoint with token and session
                let mut delete_style_resp = client.delete(format!("http://localhost:8000/token/{}/api/style/1", token).as_str())
                    .header(reqwest::header::COOKIE, cookie.clone())
                    .send()
                    .unwrap();
                assert_eq!(delete_style_resp.text().unwrap(), r#"true"#);
                assert!(delete_style_resp.status().is_success());
            }

            { // Delete user session
                let client = reqwest::Client::new();
                let mut delete_session_resp = client.delete("http://localhost:8000/api/user/session")
                    .header(reqwest::header::COOKIE, cookie.clone())
                    .send()
                    .unwrap();

                assert!(delete_session_resp.status().is_success());
                assert_eq!(delete_session_resp.text().unwrap(), "true");
            }

            { // Unable to Access the style create (FULL scope) endpoint with deleted cookie
                let err_resp = client.post("http://localhost:8000/api/style")
                    .body(r#"{
                        "name": "Awesome Style",
                        "style": "I am a style"
                    }"#)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .header(reqwest::header::COOKIE, cookie.clone())
                    .send()
                    .unwrap();
                assert_eq!(err_resp.status().as_u16(), 401);
            }
        }

        server.kill().unwrap();
    }
}
