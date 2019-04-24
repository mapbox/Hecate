use postgres::types::ToSql;
use std::io::{Error, ErrorKind};
use crate::err::HecateError;

use std::mem;

pub struct PGStream<PG: 'static> {
    eot: bool, //End of Tranmission has been sent
    cursor: String,
    pending: Option<Vec<u8>>,
    trans: postgres::transaction::Transaction<'static>,
    #[allow(dead_code)]
    conn: Box<&'static PG>
}

impl<PG: postgres::GenericConnection> std::io::Read for PGStream<PG> {
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

impl<PG: postgres::GenericConnection> PGStream<PG> {
    pub fn new(conn: &PG, cursor: String, query: String, params: &[&ToSql]) -> Result<Self, HecateError> {
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
            Err(err) => Err(HecateError::from_db(err))
        }
    }
}
