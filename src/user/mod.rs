use crate::err::HecateError;

pub fn create(conn: &impl postgres::GenericConnection, username: &String, password: &String, email: &String) -> Result<bool, HecateError> {
    match conn.query("
        INSERT INTO users (username, password, email, meta)
            VALUES ($1, crypt($2, gen_salt('bf', 10)), $3, '{}'::JSONB);
    ", &[ &username, &password, &email ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            if err.as_db().is_some() && err.as_db().unwrap().code.code() == "23505" {
                Err(HecateError::new(400, String::from("User/Email Exists"), None))
            } else {
                Err(HecateError::from_db(err))
            }
        }
    }
}

pub fn list(conn: &impl postgres::GenericConnection, limit: &Option<i16>) -> Result<serde_json::Value, HecateError> {
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
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn filter(conn: &impl postgres::GenericConnection, filter: &String, limit: &Option<i16>) -> Result<serde_json::Value, HecateError> {
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
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn set_admin(conn: &impl postgres::GenericConnection, uid: &i64) -> Result<bool, HecateError> {
    match conn.query("
        UPDATE users
            SET
                access = 'admin'
            WHERE
                id = $1
    ", &[ &uid ]) {
        Ok(_) => Ok(true),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn delete_admin(conn: &impl postgres::GenericConnection, uid: &i64) -> Result<bool, HecateError> {
    match conn.query("
        UPDATE users
            SET
                access = NULL
            WHERE
                id = $1
    ", &[ &uid ]) {
        Ok(_) => Ok(true),
        Err(err) => Err(HecateError::from_db(err))
    }
}
pub fn info(conn: &impl postgres::GenericConnection, uid: &i64) -> Result<serde_json::Value, HecateError> {
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
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn create_token(conn: &impl postgres::GenericConnection, uid: &i64) -> Result<String, HecateError> {
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
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn destroy_token(conn: &impl postgres::GenericConnection, uid: &i64, token: &String) -> Result<bool, HecateError> {
    match conn.query("
        DELETE FROM users_tokens
            WHERE
                token = $1
                AND uid = $2;
    ", &[ &token, &uid ]) {
        Ok(_) => Ok(true),
        Err(_) => Err(HecateError::new(404, String::from("Token Not Found"), None))
    }
}
