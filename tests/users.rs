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

    #[test]
    fn users() {
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
            "--auth", env::current_dir().unwrap().join("tests/fixtures/auth.default.json").to_str().unwrap()
        ]).spawn().unwrap();

        thread::sleep(Duration::from_secs(1));

        { // Seed DB with admin user to create others
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            conn.execute("
                INSERT INTO users (username, password, email)
                    VALUES ('ingalls', crypt('yeaheh', gen_salt('bf', 10)), 'ingalls@protonmail.com')
            ", &[]).unwrap();

            conn.execute("
                UPDATE users SET access = 'admin' WHERE id = 1;
            ", &[]).unwrap();
        }

        { //Create Username
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/create?username=ingalls2&password=yeaheh&email=ingalls2@protonmail.com")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Username
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/create?username=ingalls3&password=yeaheh&email=ingalls3@protonmail.com")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Username
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/create?username=filter&password=yeaheh&email=ingalls4@protonmail.com")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Username Duplicate Error
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls3@protonmail.com")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "{\"code\":400,\"reason\":\"User/Email Exists\",\"status\":\"Bad Request\"}");
            assert!(resp.status().is_client_error());
        }

        { //Feature Upload with no auth Fail
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": { "addr:housenumber": "1234", "addr:street": "Main St" },
                    "geometry": { "type": "Point", "coordinates": [ -79.46014970541, 43.67263458218963 ] }
                }"#)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();
            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "");
        }

        { //Feature Upload with bad username
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": { "addr:housenumber": "1234", "addr:street": "Main St" },
                    "geometry": { "type": "Point", "coordinates": [ -79.46014970541, 43.67263458218963 ] }
                }"#)
                .basic_auth("ingalls_bad", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "");
            assert!(resp.status().is_client_error());
        }

        { //Feature Upload with bad password
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": { "addr:housenumber": "1234", "addr:street": "Main St" },
                    "geometry": { "type": "Point", "coordinates": [ -79.46014970541, 43.67263458218963 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh2"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "");
            assert!(resp.status().is_client_error());
        }

        { //Feature Upload with correct creds
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": { "addr:housenumber": "1234", "addr:street": "Main St" },
                    "geometry": { "type": "Point", "coordinates": [ -79.46014970541, 43.67263458218963 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        // Auth with user session
        {
            // Create a new session given username & password
            let client = reqwest::Client::new();
            let mut session_resp = client.get("http://localhost:8000/api/user/session")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(session_resp.status().is_success());
            assert_eq!(session_resp.text().unwrap(), "true");

            let cookies: Vec<reqwest::cookie::Cookie> = session_resp.cookies().into_iter().collect();
            let cookie = format!("{}={}", cookies[0].name(), cookies[0].value());
            assert_eq!(cookies[0].name(), "session");
            assert!(cookies[0].value().len() > 0);

            // Access the style create (FULL scope) endpoint with cookie
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

            // Access the styles get (READ scope) endpoint with cookie
            let mut get_style_resp = client.get("http://localhost:8000/api/style/1")
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();
            assert_eq!(get_style_resp.text().unwrap(), r#"{"id":1,"name":"Awesome Style","public":false,"style":"I am a style","uid":1,"username":"ingalls"}"#);
            assert!(get_style_resp.status().is_success());

            // Create Token
            let mut token_resp = client.post("http://localhost:8000/api/user/token")
                .body(r#"{
                    "name": "JOSM Token",
                    "hours": 5
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            let json_body: serde_json::value::Value = token_resp.json().unwrap();
            assert_eq!(json_body["name"], json!("Access Token"));
            assert_eq!(json_body["uid"], json!(1));
            assert!(token_resp.status().is_success());
            let token = json_body["token"].as_str().unwrap().to_string();

            // Access the styles get (READ scope) endpoint with cookie and token
            let mut get_style_resp = client.get(format!("http://localhost:8000/token/{}/api/style/1", token).as_str())
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();
            assert_eq!(get_style_resp.text().unwrap(), r#"{"id":1,"name":"Awesome Style","public":false,"style":"I am a style","uid":1,"username":"ingalls"}"#);
            assert!(get_style_resp.status().is_success());

            // Access the style delete (FULL scope) endpoint with token and session
            let mut delete_style_resp = client.delete(format!("http://localhost:8000/token/{}/api/style/1", token).as_str())
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();
            assert_eq!(delete_style_resp.text().unwrap(), r#"true"#);
            assert!(delete_style_resp.status().is_success());

            // Delete user session
            let client = reqwest::Client::new();
            let mut delete_session_resp = client.delete("http://localhost:8000/api/user/session")
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();

            assert!(delete_session_resp.status().is_success());
            assert_eq!(delete_session_resp.text().unwrap(), "true");

            // Unable to Access the style create (FULL scope) endpoint with deleted cookie
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


        { //Test User Listing
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/users")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([{
                "id": 4,
                "access": null,
                "username": "filter",
            },{
                "id": 1,
                "access": "admin",
                "username": "ingalls",
            },{
                "id": 2,
                "access": null,
                "username": "ingalls2",
            },{
                "id": 3,
                "access": null,
                "username": "ingalls3",
            }]));
        }

        { //Test User Listing w/ Limit
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/users?limit=1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([{
                "id": 4,
                "access": null,
                "username": "filter",
            }]));
        }

        { //Test User Listing w/ Filtering
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/users?filter=in")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([{
                "id": 1,
                "access": "admin",
                "username": "ingalls",
            },{
                "id": 2,
                "access": null,
                "username": "ingalls2",
            },{
                "id": 3,
                "access": null,
                "username": "ingalls3",
            }]));
        }

        { //Test User Listing w/ Filtering & limit
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/users?filter=in&limit=2")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([{
                "id": 1,
                "access": "admin",
                "username": "ingalls",
            },{
                "id": 2,
                "access": null,
                "username": "ingalls2",
            }]));
        }

        { //Test User Listing w/ Filtering - complete name
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/users?filter=ingalls2")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([{
                "id": 2,
                "access": null,
                "username": "ingalls2",
            }]));
        }

        { //Test User Listing w/ Filtering - no match
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/users?filter=kp")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!([]));
        }

        { // Get info about my own account
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/info")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 1,
                "access": "admin",
                "username": "ingalls",
                "email": "ingalls@protonmail.com",
                "meta": {}
            }));
        }

        { // Create user to be set as admin
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/create?username=future_admin&password=yeaheh&email=fake@example.com")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { // An admin can get user info about an arbitrary user
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/6")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();

            assert_eq!(json_body, json!({
                "id": 6,
                "access": null,
                "username": "future_admin",
                "email": "fake@example.com",
                "meta": {}
            }));
        }

        { // A non-admin cannot get user info about an arbitrary user
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8000/api/user/3")
                .basic_auth("future_admin", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
        }

        { // A non-admin cannot set an admin
            let client = reqwest::Client::new();
            let resp = client.put("http://localhost:8000/api/user/1/admin")
                .basic_auth("future_admin", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
        }

        { // A non-admin cannot unset an admin
            let client = reqwest::Client::new();
            let resp = client.delete("http://localhost:8000/api/user/1/admin")
                .basic_auth("future_admin", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
        }

        { // An admin can set an admin
            let client = reqwest::Client::new();
            let resp = client.put("http://localhost:8000/api/user/6/admin")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        { // An admin can unset an admin
            let client = reqwest::Client::new();
            let resp = client.delete("http://localhost:8000/api/user/1/admin")
                .basic_auth("future_admin", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        { //Ensure admin was unset
            let client = reqwest::Client::new();
            let resp = client.get("http://localhost:8000/api/user/6")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
        }

        server.kill().unwrap();
    }
}
