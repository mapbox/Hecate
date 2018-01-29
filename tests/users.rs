extern crate curl;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use curl::easy::{Easy, List};

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

        { //Create Username
            let mut easy = Easy::new();
            easy.url("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            easy.write_function(|buf| {
                assert_eq!(buf.len(), 4);
                assert_eq!(String::from_utf8_lossy(buf), String::from("true"));
                Ok(buf.len())
            }).unwrap();
            easy.perform().unwrap();

            assert_eq!(easy.response_code(), Ok(200));
        }

        { //Duplicate Username Fail
            let mut easy = Easy::new();
            easy.url("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            easy.write_function(|buf| {
                assert_eq!(buf.len(), 90);
                assert_eq!(String::from_utf8_lossy(buf), String::from("Could not create user: duplicate key value violates unique constraint \"users_username_key\""));
                Ok(buf.len())
            }).unwrap();
            easy.perform().unwrap();

            assert_eq!(easy.response_code(), Ok(400));
        }

        { //Feature Upload with no auth Fail
            let mut easy = Easy::new();
            easy.url("http://localhost:8000/api/data/feature").unwrap();
            easy.post(true).unwrap();

            let mut list = List::new();
            list.append("Content-Type: application/json").unwrap();
            easy.http_headers(list).unwrap();

            easy.post_fields_copy(r#"
                {
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": {
                        "addr:housenumber": "1234",
                        "addr:street": "Main St"
                    },
                    "geometry": {
                        "type": "Point",
                        "coordinates": [ -79.46014970541, 43.67263458218963 ]
                    }
                }
            "#.as_bytes()).unwrap();

            easy.write_function(|buf| {
                assert_eq!(buf.len(), 95);
                assert_eq!(String::from_utf8_lossy(buf), String::from("{\"code\":401,\"reason\":\"You must be logged in to access this resource\",\"status\":\"Not Authorized\"}"));
                Ok(buf.len())
            }).unwrap();
            easy.perform().unwrap();

            assert_eq!(easy.response_code(), Ok(401));
        }

        { //Feature Upload with bad username
            let mut easy = Easy::new();
            easy.url("http://ingalls@localhost:8000/api/data/feature").unwrap();
            easy.post(true).unwrap();

            let mut list = List::new();
            list.append("Content-Type: application/json").unwrap();
            easy.http_headers(list).unwrap();

            easy.post_fields_copy(r#"
                {
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": {
                        "addr:housenumber": "1234",
                        "addr:street": "Main St"
                    },
                    "geometry": {
                        "type": "Point",
                        "coordinates": [ -79.46014970541, 43.67263458218963 ]
                    }
                }
            "#.as_bytes()).unwrap();

            easy.write_function(|buf| {
                assert_eq!(buf.len(), 15);
                assert_eq!(String::from_utf8_lossy(buf), String::from("Not Authorized!"));
                Ok(buf.len())
            }).unwrap();
            easy.perform().unwrap();

            assert_eq!(easy.response_code(), Ok(401));
        }

        { //Feature Upload with bad password
            let mut easy = Easy::new();
            easy.url("http://ingalls:yeah@localhost:8000/api/data/feature").unwrap();
            easy.post(true).unwrap();

            let mut list = List::new();
            list.append("Content-Type: application/json").unwrap();
            easy.http_headers(list).unwrap();

            easy.post_fields_copy(r#"
                {
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": {
                        "addr:housenumber": "1234",
                        "addr:street": "Main St"
                    },
                    "geometry": {
                        "type": "Point",
                        "coordinates": [ -79.46014970541, 43.67263458218963 ]
                    }
                }
            "#.as_bytes()).unwrap();

            easy.write_function(|buf| {
                assert_eq!(buf.len(), 15);
                assert_eq!(String::from_utf8_lossy(buf), String::from("Not Authorized!"));
                Ok(buf.len())
            }).unwrap();
            easy.perform().unwrap();

            assert_eq!(easy.response_code(), Ok(401));
        }

        { //Feature Upload with correct creds
            let mut easy = Easy::new();
            easy.url("http://ingalls:yeaheh@localhost:8000/api/data/feature").unwrap();
            easy.post(true).unwrap();

            let mut list = List::new();
            list.append("Content-Type: application/json").unwrap();
            easy.http_headers(list).unwrap();

            easy.post_fields_copy(r#"
                {
                    "type": "Feature",
                    "message": "Create Point",
                    "action": "create",
                    "properties": {
                        "addr:housenumber": "1234",
                        "addr:street": "Main St"
                    },
                    "geometry": {
                        "type": "Point",
                        "coordinates": [ -79.46014970541, 43.67263458218963 ]
                    }
                }
            "#.as_bytes()).unwrap();

            easy.write_function(|buf| {
                assert_eq!(buf.len(), 4);
                assert_eq!(String::from_utf8_lossy(buf), String::from("true"));
                Ok(buf.len())
            }).unwrap();
            easy.perform().unwrap();

            assert_eq!(easy.response_code(), Ok(200));
        }
    }
}
