extern crate reqwest;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use reqwest;

    #[test]
    fn bounds() {
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
                DROP DATABASE hecate;
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

        {
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();

            conn.execute(r#"
                INSERT INTO bounds(geom, name) VALUES (
                    ST_Multi(ST_SetSRID(ST_GeomFromGeoJSON('{ "type": "Polygon", "coordinates": [ [ [ -77.13363647460938, 38.83542884007305 ], [ -76.96403503417969, 38.83542884007305 ], [ -76.96403503417969, 38.974891064341726 ], [ -77.13363647460938, 38.974891064341726 ], [ -77.13363647460938, 38.83542884007305 ] ] ] }'), 4326)),
                    'dc'
                );
            "#, &[]).unwrap();
        }

        { //Create Username
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Point Inside of Bounds
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Create Point Inside of Bounds",
                    "properties": { "indc": true },
                    "geometry": { "type": "Point", "coordinates": [ -77.01210021972656,38.925763232374514 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //Create Point Outside of Bounds
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Create Point Outside of Bounds",
                    "properties": { "indc": false },
                    "geometry": { "type": "Point", "coordinates": [ -76.94755554199219,38.90385833966778 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //List Bounds
            let mut resp = reqwest::get("http://localhost:8000/api/data/bounds").unwrap();
            assert_eq!(resp.text().unwrap(), r#"["dc"]"#);
            assert!(resp.status().is_success());
        }

        { //Get Bounds
            let mut resp = reqwest::get("http://localhost:8000/api/data/bounds/dc").unwrap();
            let mut body_str = String::from(resp.text().unwrap());
            body_str.pop();
            assert_eq!(&*body_str, r#"{"id":1,"type":"Feature","version":1,"geometry":{"type":"Point","coordinates":[-77.0121002197266,38.9257632323745]},"properties":{"indc": true}}"#);
            assert!(resp.status().is_success());
        }
    }
}
