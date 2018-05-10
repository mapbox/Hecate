extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate std;
extern crate rocket;

use std::mem;

#[derive(PartialEq, Debug)]
pub enum BoundsError {
    NotFound,
    ListError(String),
    GetError(String)
}

impl BoundsError {
    pub fn to_string(&self) -> String {
        match *self {
            BoundsError::NotFound => String::from("User Not Found"),
            BoundsError::ListError(ref msg) => String::from(format!("Could not list bounds: {}", msg)),
            BoundsError::GetError(ref msg) => String::from(format!("Could not get bounds: {}", msg))
        }
    }
}

pub fn list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<Vec<String>, BoundsError> {
    match conn.query("
        SELECT name FROM bounds;
    ", &[ ]) {
        Ok(rows) => {
            let mut names = Vec::<String>::new();

            for row in rows.iter() {
                names.push(row.get(0));
            }

            Ok(names)
        },
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(BoundsError::ListError(e.message.clone())) },
                _ => Err(BoundsError::ListError(String::from("generic")))
            }
        }
    }
}

pub struct BoundsStream {
    pending: Option<Vec<u8>>,
    trans: postgres::transaction::Transaction<'static>,
    conn: Box<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>>
}

impl std::io::Read for BoundsStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut current = 0;

        while current < buf.len() {
            let mut write: Vec<u8> = Vec::new();

            if self.pending.is_some() {
                write = self.pending.clone().unwrap();
                self.pending = None;
            } else {
                let rows = self.trans.query("FETCH 1000 FROM next_bounds;", &[]).unwrap();

                if rows.len() != 0 {
                    for row_it in 0..rows.len() {
                        let feat: String = rows.get(row_it).get(0);
                        write.append(&mut feat.into_bytes().to_vec());
                        write.push(0x0A);
                    }
                }
            }

            if write.len() == 0 {
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

impl BoundsStream {
    pub fn new(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, rbounds: String) -> Result<Self, rocket::response::status::Custom<String>> {
        let pg_conn = Box::new(conn);

        let trans: postgres::transaction::Transaction = unsafe { mem::transmute(pg_conn.transaction().unwrap()) };

        trans.execute("
            DECLARE next_bounds CURSOR FOR
                SELECT
                    row_to_json(t)::TEXT
                FROM (
                    SELECT
                        geo.id AS id,
                        'Feature' AS type,
                        geo.version AS version,
                        ST_AsGeoJSON(geo.geom)::JSON AS geometry,
                        geo.props AS properties
                    FROM
                        geo,
                        bounds
                    WHERE
                        bounds.name = $1
                        AND ST_Intersects(geo.geom, bounds.geom)
                ) t
        ", &[&rbounds]).unwrap();

        Ok(BoundsStream {
            pending: None,
            trans: trans,
            conn: pg_conn
        })
    }
}
