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

        { //Create Username
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //XML Changeset Create (Node Create)
            let client = reqwest::Client::new();
            let mut resp = client.put("http://localhost:8000/api/0.6/changeset/create")
                .body(r#"<osm><changeset><tag k="created_by" v="Hecate Server"/><tag k="comment" v="Buncho Random Text"/></changeset></osm>"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "1");
            assert!(resp.status().is_success());
        }

        { //XML Changeset Verification (Node Create)
            let resp = reqwest::get("http://localhost:8000/api/delta/1").unwrap();
            assert!(resp.status().is_success());
            //TODO Check body contents
        }

        { //XML Node Create
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/0.6/changeset/1/upload")
                .body(r#"
                    <osmChange version="0.6" generator="Hecate Server">
                        <create>
                            <node id='-1' version='1' changeset='1' lat='-0.66180939203' lon='3.59219690827'>
                                <tag k='amenity' v='shop' />
                                <tag k='building' v='yes' />
                            </node>
                        </create>
                    </osmChange>
                "#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"<diffResult generator="Hecate Server" version="0.6"><node old_id="-1" new_id="1" new_version="1"/></diffResult>"#);
            assert!(resp.status().is_success());
        }

        { //XML Node Verification
            let resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert!(resp.status().is_success());
            //TODO Check body contents
        }

        { //XML Changeset Create (Node Modify)
            let client = reqwest::Client::new();
            let mut resp = client.put("http://localhost:8000/api/0.6/changeset/create")
                .body(r#"<osm><changeset><tag k="created_by" v="Hecate Server"/><tag k="comment" v="Buncho Random Text"/></changeset></osm>"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "2");
            assert!(resp.status().is_success());
        }

        { //XML Changeset Verification (Node Modify)
            let resp = reqwest::get("http://localhost:8000/api/delta/2").unwrap();
            assert!(resp.status().is_success());
            //TODO Check body contents
        }

        { //XML Node Modify
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/0.6/changeset/2/upload")
                .body(r#"
                    <osmChange version="0.6" generator="Hecate Server">
                        <modify>
                            <node id='1' version='1' changeset='1' lat='1.1' lon='1.1'>
                                <tag k='building' v='house' />
                            </node>
                        </modify>
                    </osmChange>
                "#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"<diffResult generator="Hecate Server" version="0.6"><node old_id="1" new_id="1" new_version="2"/></diffResult>"#);
            assert!(resp.status().is_success());
        }

        { //XML Node Verification
            let resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert!(resp.status().is_success());
            //TODO Check body contents
        }

        { //XML Changeset Create (Node Delete)
            let client = reqwest::Client::new();
            let mut resp = client.put("http://localhost:8000/api/0.6/changeset/create")
                .body(r#"<osm><changeset><tag k="created_by" v="Hecate Server"/><tag k="comment" v="Buncho Random Text"/></changeset></osm>"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "3");
            assert!(resp.status().is_success());
        }

        { //XML Changeset Verification (Node Delete)
            let resp = reqwest::get("http://localhost:8000/api/delta/3").unwrap();
            assert!(resp.status().is_success());
            //TODO Check body contents
        }

        { //XML Node Delete
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/0.6/changeset/3/upload")
                .body(r#"
                    <osmChange version="0.6" generator="Hecate Server">
                        <delete>
                            <node id='1' version='2' />
                        </delete>
                    </osmChange>
                "#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), r#"<diffResult generator="Hecate Server" version="0.6"><node old_id="1"/></diffResult>"#);
            assert!(resp.status().is_success());
        }

        { //XML Node Verification
            let resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert!(resp.status().is_client_error());
        }

        server.kill().unwrap();
    }
}
