extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate rocket;
extern crate base64;

extern crate serde_json;

use self::rocket::request::{self, FromRequest};
use self::rocket::http::Status;
use self::rocket::{Request, Outcome};

fn is_all(scope_type: &str, scope: &Option<String>) -> Result<bool, String> {
    match scope {
        &None => Ok(true),
        &Some(ref scope_str) => {
            match *&scope_str {
                _ => Err(format!("Scope {} must be one of public, admin, or user", scope_type)),
                "public" => Ok(true),
                "admin" => Ok(true),
                "user" => Ok(true)
            }
        }
    }
}

pub trait ValidAuth {
    fn valid(&self) -> Result<bool, String>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthSchema {
    pub get: Option<String>
}

impl ValidAuth for AuthSchema {
    fn valid(&self) -> Result<bool, String> {
        is_all("schema::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthUser {
    pub info: Option<String>,
    pub create: Option<String>,
    pub create_session: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthFeature {
    pub create: Option<String>,
    pub get: Option<String>,
    pub history: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthStyle {
    pub create: Option<String>,
    pub patch: Option<String>,
    pub set_public: Option<String>,
    pub set_private: Option<String>,
    pub delete: Option<String>,
    pub get: Option<String>,
    pub list: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthDelta {
    pub get: Option<String>,
    pub list: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthBounds {
    pub list: Option<String>,
    pub get: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthOSM {
    pub get: Option<String>,
    pub create: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomAuth {
    pub meta: Option<String>,
    pub schema: Option<AuthSchema>,
    pub user: Option<AuthUser>,
    pub feature: Option<AuthFeature>,
    pub style: Option<AuthStyle>,
    pub delta: Option<AuthDelta>,
    pub bounds: Option<AuthBounds>,
    pub osm: Option<AuthOSM>
}

impl CustomAuth {
    pub fn new() -> Self {
        CustomAuth {
            meta: Some(String::from("public")),
            schema: Some(AuthSchema {
                get: Some(String::from("public"))
            }),
            user: Some(AuthUser {
                info: Some(String::from("self")),
                create: Some(String::from("public")),
                create_session: Some(String::from("self"))
            }),
            feature: Some(AuthFeature {
                create: Some(String::from("user")),
                get: Some(String::from("public")),
                history: Some(String::from("public"))
            }),
            style: Some(AuthStyle {
                create: Some(String::from("self")),
                patch: Some(String::from("self")),
                set_public: Some(String::from("self")),
                set_private: Some(String::from("self")),
                delete: Some(String::from("self")),
                get: Some(String::from("public")),
                list: Some(String::from("public"))
            }),
            delta: Some(AuthDelta {
                get: Some(String::from("public")),
                list: Some(String::from("public"))
            }),
            bounds: Some(AuthBounds {
                list: Some(String::from("public")),
                get: Some(String::from("public"))
            }),
            osm: Some(AuthOSM {
                get: Some(String::from("public")),
                create: Some(String::from("user"))
            })
        }
    }

    pub fn valid(&self) -> Result<bool, String> {
        is_all("meta", &self.meta)?;

        Ok(true)
    }

    pub fn is_meta(&self, auth: Auth) -> bool {
        true
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
