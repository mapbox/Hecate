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
    use std::env;
    use reqwest;

    #[test]
    fn user_session_closed() {
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
        let mut server = Command::new("cargo").args(&[ "run", "--", "--auth", env::current_dir().unwrap().join("tests/fixtures/auth.default.json").to_str().unwrap()]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { // Create User
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            conn.execute("
                INSERT INTO users (username, password, email)
                    VALUES ('ingalls', crypt('yeahehyeah', gen_salt('bf', 10)), 'ingalls@protonmail.com')
            ", &[]).unwrap();
        }

        // Test UI with session & invalidation redirection
        let cookie: String;

        { // since default: user is enabled we should not be able to access the UI page
            let client = reqwest::Client::builder()
                .redirect(reqwest::RedirectPolicy::none())
                .build()
                .unwrap();

            let admin_resp = client.get("http://localhost:8000/admin/index.html")
                .send()
                .unwrap();

            assert_eq!(admin_resp.status().as_str(), "302");
            assert_eq!(admin_resp.headers().get("location").unwrap(), &"/admin/login/index.html");
        }

        { // With redirects we should get a 200 with the login page
            let client = reqwest::Client::new();

            let admin_resp = client.get("http://localhost:8000/admin/index.html")
                .send()
                .unwrap();

            assert_eq!(admin_resp.status().as_str(), "200");
        }

        { // Create a new session given username & password
            let client = reqwest::Client::new();
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

        { // With session cookie we should get the actual admin page - no redirect
            let client = reqwest::Client::builder()
                .redirect(reqwest::RedirectPolicy::none())
                .build()
                .unwrap();

            let admin_resp = client.get("http://localhost:8000/admin/index.html")
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();

            assert_eq!(admin_resp.status().as_str(), "200");
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

        { // Attempt to access admin page with expired cookie (redirects disabled)
            let client = reqwest::Client::builder()
                .redirect(reqwest::RedirectPolicy::none())
                .build()
                .unwrap();

            let admin_resp = client.get("http://localhost:8000/admin/index.html")
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();

            assert_eq!(admin_resp.status().as_str(), "302");
            assert_eq!(admin_resp.headers().get("location").unwrap(), &"/admin/login/index.html");
        }

        { // Attempt to access admin page with expired cookie (should redirect to login)
            let client = reqwest::Client::new();

            let admin_resp = client.get("http://localhost:8000/admin/index.html")
                .header(reqwest::header::COOKIE, cookie.clone())
                .send()
                .unwrap();

            assert_eq!(admin_resp.status().as_str(), "401");
        }

        server.kill().unwrap();
    }
}
