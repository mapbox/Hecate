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
///
/// By default styles are private and can only be accessed by a single user
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

/// Get the style by id, if the style is public, the user need not be logged in,
/// if the style is private ensure the owner is the requester
pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &Option<i64>, style_id: &i64) -> Result<bool, StyleError> {
    conn.query("
        SELECT
            style
        FROM
            styles
        WHERE
            id = $1
            AND (
                public IS true
                OR uid = $2
            )
    ", &[&style_id, &uid]).unwrap();

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
