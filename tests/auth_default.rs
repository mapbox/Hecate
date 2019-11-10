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

    #[test]
    fn auth_default() {
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

        { // Create User
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            conn.execute("
                INSERT INTO users (username, password, email)
                    VALUES ('ingalls', crypt('yeahehyeah', gen_salt('bf', 10)), 'ingalls@protonmail.com')
            ", &[]).unwrap();
        }

        { // Attempt to access default server
            let mut resp = reqwest::get("http://localhost:8000/").unwrap();
            assert_eq!(resp.status().as_u16(), 200);
            assert_eq!(resp.text().unwrap(), "Hello World!");
        }

        { // Attempt to access public APIs unauthenticated
            let resp = reqwest::get("http://localhost:8000/api/deltas").unwrap();

            assert_eq!(resp.status().as_u16(), 401);
        }

        { // Attempt to access public APIs
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/deltas")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());

            let json_body: serde_json::value::Value = resp.json().unwrap();
            assert_eq!(json_body, json!([]));
        }

        { // Admin UI
            let resp = reqwest::get("http://localhost:8000/admin/").unwrap();
            assert_eq!(resp.status().as_u16(), 200);
        }

        { // Attempt to create another user
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/create?username=ingalls2&password=yeahehyeah&email=ingalls2@protonmail.com")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Unauthorized\"}");
        }

        { // Set user to admin
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();

            conn.execute("
                UPDATE users SET access = 'admin' WHERE id = 1;
            ", &[]).unwrap();
        }

        { // Create another user as admin
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/user/create?username=ingalls2&password=yeahehyeah&email=ingalls2@protonmail.com")
                .basic_auth("ingalls", Some("yeahehyeah"))
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
