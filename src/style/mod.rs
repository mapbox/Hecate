extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate serde_json;

use serde_json::Value;

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
                COALESCE($1::TEXT::JSON->>'name', 'New Style')::TEXT,
                COALESCE($1::TEXT::JSON->'style', '{}'::JSON),
                $2,
                false
            );
    ", &[&style, &uid]).unwrap();

    Ok(true)
}

/// Get the style by id, if the style is public, the user need not be logged in,
/// if the style is private ensure the owner is the requester
pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &Option<i64>, style_id: &i64) -> Result<Value, StyleError> {
    match conn.query("
        SELECT
            row_to_json(t) as style
        FROM (
            SELECT
                styles.id AS id,
                styles.name AS name,
                styles.style AS style
            FROM
                styles
            WHERE
                id = $1
                AND (
                    public IS true
                    OR uid = $2
                )
        ) t
    ", &[&style_id, &uid]) {
        Ok(rows) => {
            if rows.len() != 1 {
                Err(StyleError::NotFound)
            } else {
                let style: Value = rows.get(0).get(0);

                Ok(style)
            }
        },
        Err(err) => {
            match err.as_db() {
                Some(_e) =>  Err(StyleError::NotFound),
                _ => Err(StyleError::NotFound)
            } 
        }
    }
}

pub fn update(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<bool, StyleError> {
    conn.query("
    ", &[]).unwrap();

    Err(StyleError::NotFound)
}

///Allow the owner of a given style to delete it
pub fn delete(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64, style_id: &i64) -> Result<bool, StyleError> {
    match conn.execute("
        DELETE
            FROM styles
            WHERE
                uid = $1
                AND id = $2
    ", &[&uid, &style_id]) {
        Ok(deleted) => {
            if deleted == 0 {
                Err(StyleError::NotFound)
            } else {
                Ok(true)
            }
        },
        Err(err) => {
            match err.as_db() {
                Some(_e) =>  Err(StyleError::NotFound),
                _ => Err(StyleError::NotFound)
            } 
        }
    }
}

pub fn public_list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<bool, StyleError> {
    conn.query("
        SELECT
            id,
            name
        FROM
            styles
    ", &[]).unwrap();

    Err(StyleError::NotFound)
}
