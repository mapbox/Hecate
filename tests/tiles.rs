extern crate reqwest;
extern crate postgres;
extern crate hecate;

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
    fn tiles() {
        { // Reset Database:
            let conn = Connection::connect("postgres://postgres@localhost:5432", TlsMode::None).unwrap();

            conn.execute("
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE
                    pg_stat_activity.datname = 'hecate'
                    AND pid <> pg_backend_pid();
            ", &[]).unwrap();

            conn.execute("DROP DATABASE IF EXISTS hecate;", &[]).unwrap();
            conn.execute("CREATE DATABASE hecate;", &[]).unwrap();

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
                    "message": "Creating a Point",
                    "properties": {
                        "string": "123",
                        "number": 123,
                        "array": [ 1, 2, 3 ]
                    },
                    "geometry": { "type": "Point", "coordinates": [ -97.734375,56.559482483762245 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Request a tile via API
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/tiles/1/0/0").send().unwrap();

            let mut body: Vec<u8> = Vec::new();
            resp.read_to_end(&mut body).unwrap();

            assert_eq!(body.len(), 100);
            assert!(resp.status().is_success());
        }

        { //Ensure Tile was places in DB tilecache
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            let res = conn.query("SELECT ref, tile FROM tiles", &[]).unwrap();

            assert_eq!(res.len(), 2);

            let tile_ref: String = res.get(0).get(0);
            assert_eq!(tile_ref, String::from("1/0/0"));

            let tile: Vec<u8> = res.get(0).get(1);

            assert_eq!(tile.len(), 65);
        }

        { //Request a tile via API
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/tiles/1/0/0").send().unwrap();

            let mut body: Vec<u8> = Vec::new();
            resp.read_to_end(&mut body).unwrap();

            assert_eq!(body.len(), 100);
            assert!(resp.status().is_success());
        }

        { //Request a tile regen - unauthenticated
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/tiles/1/0/0/regen").send().unwrap();

            assert_eq!(resp.text().unwrap(), "{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Unauthorized\"}");
            assert!(resp.status().is_client_error());
        }

        { //Request a tile meta
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/tiles/1/0/0/meta").send().unwrap();

            assert!(resp.text().unwrap().contains("created"));
            assert!(resp.status().is_success());
        }

        { //Request a tile regen - authenticated
            let client = reqwest::Client::new();
            let mut resp = client.get("http://localhost:8000/api/tiles/1/0/0/regen")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            let mut body: Vec<u8> = Vec::new();
            resp.read_to_end(&mut body).unwrap();

            assert_eq!(body.len(), 100);
            assert!(resp.status().is_success());
        }

        {
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();

            conn.execute("
                UPDATE users SET access = 'admin' WHERE id = 1;
            ", &[]).unwrap();
        }

        { //Wipe Tile DB
            let client = reqwest::Client::new();
            let resp = client.delete("http://localhost:8000/api/tiles")
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();

            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
