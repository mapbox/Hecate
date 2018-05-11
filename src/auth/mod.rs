extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate rocket;
extern crate base64;

extern crate serde_json;

use self::rocket::request::{self, FromRequest};
use self::rocket::http::Status;
use self::rocket::{Request, Outcome};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthSchema {
    get: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthUser {
    info: Option<String>,
    create: Option<String>,
    create_session: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthFeature {
    create: Option<String>,
    get: Option<String>,
    history: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthStyle {
    create: Option<String>,
    patch: Option<String>,
    set_public: Option<String>,
    set_private: Option<String>,
    delete: Option<String>,
    get: Option<String>,
    list: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthDelta {
    get: Option<String>,
    list: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthBounds {
    list: Option<String>,
    get: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthOSM {
    get: Option<String>,
    create: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomAuth {
    meta: Option<String>,
    schema: Option<AuthSchema>,
    user: Option<AuthUser>,
    feature: Option<AuthFeature>,
    style: Option<AuthStyle>,
    delta: Option<AuthDelta>,
    bounds: Option<AuthBounds>,
    osm: Option<AuthOSM>
}

impl CustomAuth {
    pub fn new() -> Self {
        CustomAuth {
            meta: Some(String::from("public")),
            schema: None,
            user: None,
            feature: None,
            style: None,
            delta: None,
            bounds: None,
            osm: None
        }
    }
}

pub struct Auth {
    token: Option<String>,
    basic: Option<(String, String)>
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
                if res.len() != 1 { return None; }
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

        if keys.len() != 1 || keys[0].len() < 7 {
            return Outcome::Success(Auth {
                token: None,
                basic: None
            });
        }

        let mut authtype = String::from(keys[0]);
        let auth = authtype.split_off(6);

        if authtype != "Basic " {
            return Outcome::Success(Auth {
                token: None,
                basic: None
            });
        }

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
