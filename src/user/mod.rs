extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate rocket;
extern crate base64;
extern crate serde_json;

#[derive(PartialEq, Debug)]
pub enum UserError {
    NotFound,
    NotAuthorized,
    ListError(String),
    AdminError(String),
    CreateError(String),
    CreateTokenError(String)
}

impl UserError {
    pub fn to_string(&self) -> String {
        match *self {
            UserError::NotFound => String::from("User Not Found"),
            UserError::NotAuthorized => String::from("User Not Authorized"),
            UserError::ListError(ref msg) => String::from(format!("Could not set list users: {}", msg)),
            UserError::AdminError(ref msg) => String::from(format!("Could not set admin on user: {}", msg)),
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

pub fn list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, limit: &Option<i16>) -> Result<serde_json::Value, UserError> {
    let limit: i16 = match limit {
        None => 100,
        Some(limit) => if *limit > 100 { 100 } else { *limit }
    };

    match conn.query("
        SELECT 
            COALESCE(json_agg(row_to_json(row)), '[]'::JSON)
        FROM (
            SELECT
                id,
                access,
                username
            FROM
                users
            ORDER BY
                username
            LIMIT $1::SmallInt
        ) row;
    ", &[ &limit ]) {
        Ok(rows) => Ok(rows.get(0).get(0)),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::CreateError(e.message.clone())) },
                _ => Err(UserError::CreateError(String::from("generic")))
            }
        }
    }
}

pub fn filter(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, filter: &String, limit: &Option<i16>) -> Result<serde_json::Value, UserError> {
    let limit: i16 = match limit {
        None => 100,
        Some(limit) => if *limit > 100 { 100 } else { *limit }
    };

    match conn.query("
        SELECT 
            COALESCE(json_agg(row_to_json(row)), '[]'::JSON)
        FROM (
            SELECT
                id,
                access,
                username
            FROM
                users
            WHERE
                username ~ $1
            ORDER BY
                username
            LIMIT $2::SmallInt
        ) row;
    ", &[ &filter, &limit ]) {
        Ok(rows) => Ok(rows.get(0).get(0)),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::CreateError(e.message.clone())) },
                _ => Err(UserError::CreateError(String::from("generic")))
            }
        }
    }
}

pub fn set_admin(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64) -> Result<bool, UserError> {
    match conn.query("
        UPDATE users
            SET
                access = 'admin'
            WHERE
                id = $1
    ", &[ &uid ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::AdminError(e.message.clone())) },
                _ => Err(UserError::AdminError(String::from("generic")))
            }
        }
    }
}

pub fn delete_admin(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64) -> Result<bool, UserError> {
    match conn.query("
        UPDATE users
            SET
                access = NULL
            WHERE
                id = $1
    ", &[ &uid ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::AdminError(e.message.clone())) },
                _ => Err(UserError::AdminError(String::from("generic")))
            }
        }
    }
}
pub fn info(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64) -> Result<serde_json::Value, UserError> {
    match conn.query("
        SELECT row_to_json(u)
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
        Ok(res) => Ok(res.get(0).get(0)),
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

pub fn destroy_token(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, uid: &i64, token: &String) -> Result<bool, UserError> {
    match conn.query("
        DELETE FROM users_tokens
            WHERE
                token = $1
                AND uid = $2;
    ", &[ &token, &uid ]) {
        Ok(_) => Ok(true),
        Err(_) => Err(UserError::NotFound)
    }
}
