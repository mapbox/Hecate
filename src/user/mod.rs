extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate bcrypt;

use self::bcrypt::{hash, verify};

#[derive(PartialEq, Debug)]
pub enum UserError {
    NotFound,
    CreateError(String)
}

impl UserError {
    pub fn to_string(&self) -> String {
        match *self {
            UserError::NotFound => String::from("User Not Found"),
            UserError::CreateError(ref msg) => String::from(format!("Could not create user: {}", msg))
        }
    }
}

pub fn create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, username: &String, password: &String, email: &String) -> Result<bool, UserError> {
    let hashed = match hash(&password, 12) {
        Ok(hash) => hash,
        Err(_) => { return Err(UserError::CreateError(String::from("Failed to hash password"))); }
    };

    match conn.query("
        INSERT INTO users (username, password, email, meta) VALUES ($1, $2, $3, '{}'::JSONB);
    ", &[ &username, &hashed, &email ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::CreateError(e.message.clone())) },
                _ => Err(UserError::CreateError(String::from("generic")))
            }
        }
    }
}

pub fn auth(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, username: &String, password: &String) -> Result<Option<i64>, UserError> {
    match conn.query("
        SELECT id, password FROM users WHERE username = $1;
    ", &[ &username ]) {
        Ok(res) => {
            let uid: i64 = res.get(0).get(0);
            let stored: String = res.get(0).get(1);

            match verify(&password, &stored) {
                Ok(_) => Ok(Some(uid)),
                Err(_) => Ok(None)
            }
        },
        Err(_) => Err(UserError::NotFound)
    }
}
