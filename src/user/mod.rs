extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate rocket;
extern crate base64;

use self::rocket::request::{self, FromRequest};
use self::rocket::http::Status;
use self::rocket::{Request, Outcome};


pub struct Auth {
    token: Option<String>,
    basic: Option<(String, String)>
}

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

pub fn auth(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, auth: Auth) -> Option<i64> {
    if auth.basic != None {
        let (username, password): (String, String) = auth.basic.unwrap();

        match conn.query("
            SELECT id
                FROM users
                WHERE
                    username = $1
                    AND password = crypt($2, password)
        ", &[ &username, &password ]) {
            Ok(res) => {
                if res.len() == 0 { return None; }
                let uid: i64 = res.get(0).get(0);

                Some(uid)
            },
            Err(_) => None
        }
    } else if auth.token != None {
        let token: String = auth.token.unwrap();

        match conn.query("
            SELECT uid
            FROM users_tokens
            WHERE
                token = $1
                AND now() < expiry
        ", &[ &token ]) {
            Ok(res) => {
                if res.len() == 0 { return None; }
                let uid: i64 = res.get(0).get(0);

                Some(uid)
            },
            Err(_) => None
        }
    } else {
        None
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
        Ok(res) => {
            Ok(true)
        },
        Err(_) => Err(UserError::NotFound)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Auth, ()> {
        match request.cookies().get_private("session") {
            Some(token) => {
                return Outcome::Success(Auth {
                    token: Some(String::from(token.value())),
                    basic: None
                });
            },
            None => ()
        };

        let keys: Vec<_> = request.headers().get("Authorization").collect();

        if keys.len() != 1 || keys[0].len() < 7 { return Outcome::Failure((Status::Unauthorized, ())); }

        let mut authtype = String::from(keys[0]);
        let auth = authtype.split_off(6);

        if authtype != "Basic " { return Outcome::Failure((Status::Unauthorized, ())); }

        match base64::decode(&auth) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(decoded_str) => {

                    let split = decoded_str.split(":").collect::<Vec<&str>>();

                    if split.len() != 2 { return Outcome::Failure((Status::Unauthorized, ())); }

                    Outcome::Success(Auth {
                        token: None,
                        basic: Some((String::from(split[0]), String::from(split[1])))
                    })
                },
                Err(_) => Outcome::Failure((Status::Unauthorized, ()))
            },
            Err(_) => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
