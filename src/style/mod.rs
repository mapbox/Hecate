extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate serde_json;

#[derive(PartialEq, Debug)]
pub enum StyleError {
    NotFound,
}

impl StyleError {
    pub fn to_string(&self) -> String {
        match *self {
            StyleError::NotFound => String::from("Style Not Found"),
        }
    }
}

/// Creates a new GL JS Style under a given user account
/// By default styles are private and can only be accessed
/// by a single user
pub fn create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64, style: &String) -> Result<bool, StyleError> {
    conn.query("
        INSERT into styles (name, style, uid, public)
            VALUES (
                'New Style',
                $1,
                $2,
                false
            )
    ", &[&style, &uid]).unwrap();

    Ok(true)
}

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<bool, StyleError> {
    conn.query("
    ", &[]).unwrap();

    Err(StyleError::NotFound)
}

pub fn update(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<bool, StyleError> {
    conn.query("
    ", &[]).unwrap();

    Err(StyleError::NotFound)
}

pub fn delete(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<bool, StyleError> {
    conn.query("
    ", &[]).unwrap();

    Err(StyleError::NotFound)
}
pub fn list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<bool, StyleError> {
    conn.query("
    ", &[]).unwrap();

    Err(StyleError::NotFound)
}
