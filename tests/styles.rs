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
    fn styles() {
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
            "--database_read", "hecate_read@localhost:5432/hecate"
        ]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { //Create Username (ingalls)
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Style
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/style")
                .body(r#"{
                    "name": "Awesome Style",
                    "style": "I am a style"
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "1");
            assert!(resp.status().is_success());
        }

        { //Get Style - No Auth
            let mut resp = reqwest::get("http://localhost:8000/api/style/1").unwrap();
            assert_eq!(resp.text().unwrap(), "\"Style Not Found\"");
            assert!(resp.status().is_client_error());
        }

        { //Get Style - Authed: ingalls
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/style/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"{"id":1,"name":"Awesome Style","public":false,"style":"I am a style","uid":1,"username":"ingalls"}"#);
            assert!(resp.status().is_success());
        }

        { //Get Non-Existant Style - Auth: ingalls
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/style/100")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "\"Style Not Found\"");
            assert!(resp.status().is_client_error());
        }

        { //Update Style Name - Auth: ingalls
            let client = reqwest::Client::new();
            let mut resp = client.patch("http://localhost:8000/api/style/1")
                .body(r#"{
                    "name": "Modified Awesome Style"
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Get Style - Authed: ingalls
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/style/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"{"id":1,"name":"Modified Awesome Style","public":false,"style":"I am a style","uid":1,"username":"ingalls"}"#);
            assert!(resp.status().is_success());
        }


        { //Update Style Style - Auth: ingalls
            let client = reqwest::Client::new();
            let mut resp = client.patch("http://localhost:8000/api/style/1")
                .body(r#"{
                    "style": "I am a modified style"
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Get Style - Authed: ingalls
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/style/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"{"id":1,"name":"Modified Awesome Style","public":false,"style":"I am a modified style","uid":1,"username":"ingalls"}"#);
            assert!(resp.status().is_success());
        }

        { //Delete Style - No Auth
            let client = reqwest::Client::new();
            let mut resp = client.delete("http://localhost:8000/api/style/1")
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Not Authorized\"}");
            assert!(resp.status().is_client_error());
        }

        { //Delete Ingalls - ingalls
            let client = reqwest::Client::new();
            let mut resp = client.delete("http://localhost:8000/api/style/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"true"#);
            assert!(resp.status().is_success());
        }
        
        { //Delete Style - Doesnt Exist
            let client = reqwest::Client::new();
            let mut resp = client.delete("http://localhost:8000/api/style/100")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "\"Style Not Found\"");
            assert!(resp.status().is_client_error());
        }

        { //Create Style 1 For List Tests
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/style")
                .body(r#"{
                    "name": "Style 1",
                    "style": "I am a style"
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "2");
            assert!(resp.status().is_success());
        }

        { //Create Style 2 For List Tests
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/style")
                .body(r#"{
                    "name": "Style 2",
                    "style": "I am a style"
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "3");
            assert!(resp.status().is_success());
        }

        { //Get List of Public Styles
            let mut resp = reqwest::get("http://localhost:8000/api/styles").unwrap();
            assert_eq!(resp.text().unwrap(), "[]");
            assert!(resp.status().is_success());
        }

        { //Mark Style 1 as Public
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/style/2/public")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Get Public Style - No Auth
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/style/2")
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"{"id":2,"name":"Style 1","public":true,"style":"I am a style","uid":1,"username":"ingalls"}"#);
            assert!(resp.status().is_success());
        }

        { //Get List of Public Styles - Style 1 should now appear, since it is now public
            let mut resp = reqwest::get("http://localhost:8000/api/styles").unwrap();
            assert_eq!(resp.text().unwrap(), r#"[{"id":2,"name":"Style 1","public":true,"uid":1,"username":"ingalls"}]"#);
            assert!(resp.status().is_success());
        }

        { //Get User List of Public Styles - Style 1 should now appear, since it is now public
            let mut resp = reqwest::get("http://localhost:8000/api/styles/1").unwrap();
            assert_eq!(resp.text().unwrap(), r#"[{"id":2,"name":"Style 1","public":true,"uid":1,"username":"ingalls"}]"#);
            assert!(resp.status().is_success());
        }

        { //Get User List of All Styles - authed user checking their own styles should see all
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/styles/1")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"[{"id":2,"name":"Style 1","public":true,"uid":1,"username":"ingalls"},{"id":3,"name":"Style 2","public":false,"uid":1,"username":"ingalls"}]"#);
            assert!(resp.status().is_success());
        }

        { //Mark Style 1 as Private again
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/style/2/private")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Get List of Public Styles - Style 1 should not appear as it is private again
            let mut resp = reqwest::get("http://localhost:8000/api/styles").unwrap();
            assert_eq!(resp.text().unwrap(), "[]");
            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
