extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;

#[derive(PartialEq, Debug)]
pub enum UserError {
    NotFound,
    CreateError(String),
    CreateTokenError(String)
}

impl UserError {
    pub fn to_string(&self) -> String {
        match *self {
            UserError::NotFound => String::from("User Not Found"),
            UserError::CreateError(ref msg) => String::from(format!("Could not create user: {}", msg)),
            UserError::CreateTokenError(ref msg) => String::from(format!("Could not create token: {}", msg))
        }
    }
}

pub fn create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, username: &String, password: &String, email: &String) -> Result<bool, UserError> {
    match conn.query("
        INSERT INTO users (username, password, email, meta) VALUES ($1, crypt($2, gen_salt('bf', 10)), $3, '{}'::JSONB);
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

pub fn auth(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, username: &String, password: &String) -> Result<Option<i64>, UserError> {
    match conn.query("
        SELECT
            id
        FROM
            users
        WHERE
            username = $1
            AND password = crypt($2, password)
    ", &[ &username, &password ]) {
        Ok(res) => {
            if res.len() == 0 { return Ok(None); }
            let uid: i64 = res.get(0).get(0);

            Ok(Some(uid))
        },
        Err(_) => Err(UserError::NotFound)
    }
}

pub fn create_token(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64) -> Result<String, UserError> {
    match conn.query("
        INSERT INTO users_tokens (uid, token, expiry)
            VALUES (
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

/*
pub fn destroy_token(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, username: &String) -> Result<String, UserError> {
    match conn.query("
        SELECT
            id
        FROM
            users
        WHERE
            username = $1
            AND password = crypt($2, password)
    ", &[ &username, &password ]) {
        Ok(res) => {
            if res.len() == 0 { return Ok(None); }
            let uid: i64 = res.get(0).get(0);

            Ok(Some(uid))
        },
        Err(_) => Err(UserError::NotFound)
    }
}

pub fn auth_token(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, username: &String) -> Result<String, UserError> {
    match conn.query("
        SELECT
            id
        FROM
            users
        WHERE
            username = $1
            AND password = crypt($2, password)
    ", &[ &username, &password ]) {
        Ok(res) => {
            if res.len() == 0 { return Ok(None); }
            let uid: i64 = res.get(0).get(0);

            Ok(Some(uid))
        },
        Err(_) => Err(UserError::NotFound)
    }
}
*/
