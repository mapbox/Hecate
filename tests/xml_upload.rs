extern crate reqwest;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use reqwest;

    #[test]
    fn xml_upload() {
        { // Reset Database:
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
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //XML Changeset Create
            let client = reqwest::Client::new();
            let mut resp = client.put("http://localhost:8000/api/0.6/changeset/create")
                .body(r#"<osm><changeset><tag k="created_by" v="Hecate Server"/><tag k="comment" v="Buncho Random Text"/></changeset></osm>"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "1");
            assert!(resp.status().is_success());
        }
    }
}
