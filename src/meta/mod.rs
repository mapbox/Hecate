extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate serde_json;
extern crate std;
extern crate rocket;

#[derive(PartialEq, Debug)]
pub enum MetaError {
    NotFound,
    ListError(String),
    GetError(String),
    SetError(String)
}

impl MetaError {
    pub fn to_string(&self) -> String {
        match *self {
            MetaError::NotFound => String::from("Key/Value Not Found"),
            MetaError::ListError(ref msg) => String::from(format!("Could not list Key/Value: {}", msg)),
            MetaError::GetError(ref msg) => String::from(format!("Could not get Key/Value: {}", msg)),
            MetaError::SetError(ref msg) => String::from(format!("Could not set Key/Value: {}", msg))
        }
    }
}

pub fn list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<Vec<String>, MetaError> {
    match conn.query("
        SELECT key FROM meta ORDER BY key
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
                Some(e) => { Err(MetaError::ListError(e.message.clone())) },
                _ => Err(MetaError::ListError(String::from("generic")))
            }
        }
    }
}

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, key: &String) -> Result<serde_json::Value, MetaError> {
    match conn.query("
        SELECT value::JSON FROM meta WHERE key = $1;
    ", &[ &key ]) {
        Ok(rows) => {
            if rows.len() == 0 {
                Ok(json!(null))
            } else {
                Ok(rows.get(0).get(0))
            }
        },
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(MetaError::GetError(e.message.clone())) },
                _ => Err(MetaError::GetError(String::from("generic")))
            }
        }
    }
}

pub fn set(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, key: &String, value: &serde_json::Value) -> Result<bool, MetaError> {
    match conn.query("
        INSERT INTO meta (key, value) VALUES ($1, $2)
            ON CONFLICT (key) DO
                UPDATE
                    SET value = $2
                    WHERE meta.key = $1
    ", &[ &key, &value ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(MetaError::SetError(e.message.clone())) },
                _ => Err(MetaError::SetError(String::from("generic")))
            }
        }
    }
}
