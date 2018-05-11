extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate rocket;
extern crate base64;

use self::rocket::request::{self, FromRequest};
use self::rocket::http::Status;
use self::rocket::{Request, Outcome};

#[derive(PartialEq, Debug)]
pub enum UserError {
    NotFound,
    NotAuthorized,
    CreateError(String),
    CreateTokenError(String)
}

impl UserError {
    pub fn to_string(&self) -> String {
        match *self {
            UserError::NotFound => String::from("User Not Found"),
            UserError::NotAuthorized => String::from("User Not Authorized"),
            UserError::CreateError(ref msg) => String::from(format!("Could not create user: {}", msg)),
            UserError::CreateTokenError(ref msg) => String::from(format!("Could not create token: {}", msg))
        }
    }
}

pub fn create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, username: &String, password: &String, email: &String) -> Result<bool, UserError> {
    match conn.query("
        INSERT INTO users (username, password, email, meta)
            VALUES ($1, crypt($2, gen_salt('bf', 10)), $3, '{}'::JSONB);
    ", &[ &username, &password, &email ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::CreateError(e.message.clone())) },
                _ => Err(UserError::CreateError(String::from("generic")))
            }
        }
    }
}

pub fn info(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64) -> Result<String, UserError> {
    match conn.query("
        SELECT row_to_json(u)::TEXT
        FROM (
            SELECT
                id,
                username,
                email,
                meta
            FROM
                users
            WHERE id = $1
        ) u
    ", &[ &uid ]) {
        Ok(res) => {
            let info: String = res.get(0).get(0);
            Ok(info)
        },
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::CreateTokenError(e.message.clone())) },
                _ => Err(UserError::CreateTokenError(String::from("generic")))
            }
        }
    }
}

pub fn create_token(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64) -> Result<String, UserError> {
    match conn.query("
        INSERT INTO users_tokens (name, uid, token, expiry)
            VALUES (
                'Session Token',
                $1,
                md5(random()::TEXT),
                now() + INTERVAL '4 hours'
            )
            RETURNING token;
    ", &[ &uid ]) {
        Ok(res) => {
            let token: String = res.get(0).get(0);
            Ok(token)
        },
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::CreateTokenError(e.message.clone())) },
                _ => Err(UserError::CreateTokenError(String::from("generic")))
            }
        }
    }
}

pub fn destroy_token(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, token: &String) -> Result<bool, UserError> {
    match conn.query("
        DELETE FROM users_tokens
            WHERE token = $1;
    ", &[ &token ]) {
        Ok(_) => Ok(true),
        Err(_) => Err(UserError::NotFound)
    }
}
