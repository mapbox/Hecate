use postgres::types::ToSql;
use std::io::{Error, ErrorKind};
use crate::err::HecateError;
use bytes::Bytes;
use futures::Async;

use std::mem;

pub struct PGStream {
    eot: bool, //End of Tranmission has been sent
    cursor: String,
    pending: Option<Vec<u8>>,
    trans: postgres::transaction::Transaction<'static>,
    #[allow(dead_code)]
    conn: Box<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>>
}

impl futures::stream::Stream for PGStream {
    type Item = Bytes;
    type Error = HecateError;

    fn poll(&mut self) -> Result<futures::Async<Option<Self::Item>>, Self::Error> {
        let rows = match self.trans.query(&*format!("FETCH 1000 FROM {};", &self.cursor), &[]) {
            Ok(rows) => rows,
            Err(err) => { return Err(HecateError::new(500, err.to_string(), None)); }
        };

        if rows.len() == 0 {
            if self.eot {
                // The Stream is complete
                return Ok(Async::Ready(None));
            } else {
                self.eot = true;
                // Write EOD Character to Stream
                return Ok(Async::Ready(Some(Bytes::from(String::from("0x04")))));
            }
        }

        let mut feats = String::new();

        for row_it in 0..rows.len() {
            let feat: String = rows.get(row_it).get(0);
            feats.push_str(&*feat);
            feats.push('\n');
        }

        Ok(Async::Ready(Some(Bytes::from(feats))))
    }
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
    pub fn new(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, cursor: String, query: String, params: &[&dyn ToSql]) -> Result<Self, HecateError> {
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

