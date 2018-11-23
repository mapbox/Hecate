extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate std;
extern crate rocket;
extern crate serde_json;

use postgres::types::ToSql;
use rocket::response::status;
use rocket::http::Status as HTTPStatus;
use rocket_contrib::json::Json;
use std::io::{Error, ErrorKind};

use std::mem;

pub struct PGStream {
    eot: bool, //End of Tranmission has been sent
    cursor: String,
    pending: Option<Vec<u8>>,
    trans: postgres::transaction::Transaction<'static>,
    #[allow(dead_code)]
    conn: Box<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>>
}

impl std::io::Read for PGStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut current = 0;

        while current < buf.len() {
            let mut write: Vec<u8> = Vec::new();

            if self.pending.is_some() {
                write = self.pending.clone().unwrap();
                self.pending = None;
            } else {
                let rows = match self.trans.query(&*format!("FETCH 1000 FROM {};", &self.cursor), &[]) {
                    Ok(rows) => rows,
                    Err(err) => {
                        return Err(Error::new(ErrorKind::Other, format!("{:?}", err)))
                    }
                };

                if rows.len() != 0 {
                    for row_it in 0..rows.len() {
                        let feat: String = rows.get(row_it).get(0);
                        write.append(&mut feat.into_bytes().to_vec());
                        write.push(0x0A);
                    }
                }
            }

            if write.len() == 0 && !self.eot {
                write.push(0x04); //Write EOT Character To Stream
                self.eot = true;
            }

            if write.len() == 0 && self.eot {
                //No more data to fetch, close up shop
                break;
            } else if current + write.len() > buf.len() {
                //There is room to put a partial feature, saving the remaining
                //to the pending q and ending

                for it in current..buf.len() {
                    buf[it] = write[it - current];
                }

                let pending = write[buf.len() - current..write.len()].to_vec();
                self.pending = Some(pending);

                current = current + (buf.len() - current);

                break;
            } else {
                //There is room in the buff to print the whole feature
                //and iterate around to grab another

                for it in 0..write.len() {
                    buf[current + it] = write[it];
                }

                current = current + write.len();
            }
        }

        Ok(current)
    }
}

impl PGStream {
    pub fn new(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, cursor: String, query: String, params: &[&ToSql]) -> Result<Self, rocket::response::status::Custom<Json<serde_json::Value>>> {
        let pg_conn = Box::new(conn);

        let trans: postgres::transaction::Transaction = unsafe {
            mem::transmute(pg_conn.transaction().unwrap())
        };

        match trans.execute(&*query, params) {
            Ok(_) => {
                Ok(PGStream {
                    eot: false,
                    cursor: cursor,
                    pending: None,
                    trans: trans,
                    conn: pg_conn
                })
            },
            Err(err) => {
                Err(status::Custom(HTTPStatus::ServiceUnavailable, Json(json!({
                    "code": 500,
                    "status": "Internal Server Error",
                    "reason": format!("{:?}", err)
                }))))
            }
        }
    }
}
