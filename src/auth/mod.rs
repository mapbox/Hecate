use crate::err::HecateError;
use actix_http::httpmessage::HttpMessage;

mod config;
mod middleware;
pub use config::AuthContainer;
pub use config::ValidAuth;
pub use config::CustomAuth;

#[derive(Debug, PartialEq, Clone)]
pub enum AuthDefault {
    Public,
    User,
    Admin
}

#[derive(Debug, PartialEq, Clone)]
pub struct Auth {
    pub uid: Option<i64>,
    pub access: Option<String>,
    pub token: Option<String>,
    pub basic: Option<(String, String)>
}

impl Auth {
    pub fn new() -> Self {
        Auth {
            uid: None,
            access: None,
            token: None,
            basic: None
        }
    }

    ///
    /// Remove user data from the Auth object
    ///
    /// Used as a generic function by validate to ensure future
    /// authentication methods are cleared with each validate
    ///
    pub fn secure(&mut self, user: Option<(i64, Option<String>)>) {
        match user {
            Some(user) => {
                self.uid = Some(user.0);
                self.access = user.1;
            }
            _ => ()
        }
        self.token = None;
        self.basic = None;
    }

    ///
    /// The Rocket Request guard simply provides a utility wrapper from the request to a more
    /// easily parsable auth object. It **does not** perform any authentication.
    ///
    /// This function takes the populated Auth struct and checks if the token/basic auth is valid,
    /// populated the uid field
    ///
    /// Note: Once validated the token/basic auth used to validate the user will be set to null
    ///
    pub fn validate(&mut self, conn: &impl postgres::GenericConnection) -> Result<Option<i64>, HecateError> {
        if self.basic.is_some() {
            match conn.query("
                SELECT
                    id,
                    access
                FROM users
                WHERE
                    username = $1
                    AND password = crypt($2, password)
            ", &[ &self.basic.as_ref().unwrap().0 , &self.basic.as_ref().unwrap().1 ]) {
                Ok(res) => {
                    if res.len() != 1 {
                        return Err(config::not_authed());
                    }

                    let uid: i64 = res.get(0).get(0);
                    let access: Option<String> = res.get(0).get(1);

                    self.secure(Some((uid, access)));

                    return Ok(Some(uid));
                },
                _ => {
                    return Err(config::not_authed());
                }
            }
        } else if self.token.is_some() {
            match conn.query("
                SELECT
                    users_tokens.uid,
                    users.access
                FROM
                    users_tokens,
                    users
                WHERE
                    token = $1
                    AND now() < expiry
                    AND users_tokens.uid = users.id
            ", &[ &self.token.as_ref().unwrap() ]) {
                Ok(res) => {
                    if res.len() == 0 {
                        return Err(config::not_authed());
                    }

                    let uid: i64 = res.get(0).get(0);
                    let access: Option<String> = res.get(0).get(1);

                    self.secure(Some((uid, access)));

                    return Ok(Some(uid));
                },
                _ => {
                    return Err(config::not_authed());
                }
            }
        }

        Ok(None)
    }
}

impl actix_web::FromRequest for Auth {
    type Error = HecateError;
    type Future = Result<Self, Self::Error>;
    type Config = ();

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let mut auth = Auth::new();

        match req.cookie("session") {
            Some(token) => {
                auth.token = Some(String::from(token.value()));

                return Ok(auth);
            },
            None => ()
        };


        match req.headers().get("Authorization") {
            Some(key) => {
                if key.len() < 7 {
                    return Ok(auth);
                }

                let mut authtype = match key.to_str() {
                    Ok(key) => key.to_string(),
                    Err(_) => { return Err(HecateError::new(401, String::from("Unauthorized"), None)); }
                };
                let auth_str = authtype.split_off(6);

                if authtype != "Basic " {
                    return Ok(auth);
                }

                match base64::decode(&auth_str) {
                    Ok(decoded) => match String::from_utf8(decoded) {
                        Ok(decoded_str) => {

                            let split = decoded_str.split(":").collect::<Vec<&str>>();

                            if split.len() != 2 { return Err(HecateError::new(401, String::from("Unauthorized"), None)); }

                            auth.basic = Some((String::from(split[0]), String::from(split[1])));

                            return Ok(auth);
                        },
                        Err(_) => { return Err(HecateError::new(401, String::from("Unauthorized"), None)); }
                    },
                    Err(_) => { return Err(HecateError::new(401, String::from("Unauthorized"), None)); }
                }
            },
            None => ()
        };

        Ok(auth)
    }
}


