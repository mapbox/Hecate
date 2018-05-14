extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate rocket;
extern crate base64;

extern crate serde_json;

use self::rocket::request::{self, FromRequest};
use self::rocket::http::Status;
use self::rocket::{Request, Outcome};
use self::rocket::response::status;

///
/// Allows a category to be null, public, admin, or user
///
/// This category makes up the majority of endpoints in hecate and is the most
/// flexible
///
fn is_all(scope_type: &str, scope: &Option<String>) -> Result<bool, String> {
    match scope {
        &None => Ok(true),
        &Some(ref scope_str) => {
            match scope_str as &str {
                "public" => Ok(true),
                "admin" => Ok(true),
                "user" => Ok(true),
                _ => Err(format!("Scope {} must be one of 'public', 'admin', 'user', or null", scope_type)),
            }
        }
    }
}

///
/// Allows a category to be null, self, or admin
///
/// This category is used for CRUD operations against data for a specfic user,
/// not only must the user be logged in but the user can only update their own
/// data
///
fn is_self(scope_type: &str, scope: &Option<String>) -> Result<bool, String> {
    match scope {
        &None => Ok(true),
        &Some(ref scope_str) => {
            match scope_str as &str {
                "self" => Ok(true),
                "admin" => Ok(true),
                _ => Err(format!("Scope {} must be one of 'self', 'admin', or null", scope_type)),
            }
        }
    }
}

///
/// Allows a category to be null, user, or admin
///
/// This category is used primarily for feature operations. The user must be
/// logged in but can make changes to any feature, including features created
/// by another user
///
fn is_auth(scope_type: &str, scope: &Option<String>) -> Result<bool, String> {
    match scope {
        &None => Ok(true),
        &Some(ref scope_str) => {
            match scope_str as &str {
                "user" => Ok(true),
                "admin" => Ok(true),
                _ => Err(format!("Scope {} must be one of 'self', 'admin', or null", scope_type)),
            }
        }
    }
}

pub trait ValidAuth {
    fn is_valid(&self) -> Result<bool, String>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthSchema {
    pub get: Option<String>
}

impl ValidAuth for AuthSchema {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("schema::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthMVT {
    pub get: Option<String>,
    pub regen: Option<String>,
    pub meta: Option<String>
}

impl ValidAuth for AuthMVT {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("mvt::get", &self.get)?;
        is_all("mvt::regen", &self.regen)?;
        is_all("mvt::meta", &self.meta)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthUser {
    pub info: Option<String>,
    pub create: Option<String>,
    pub create_session: Option<String>
}

impl ValidAuth for AuthUser {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("user::create", &self.create)?;

        is_self("user::create_session", &self.create_session)?;
        is_self("user::info", &self.info)?;

        Ok(true)
    }
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

impl ValidAuth for AuthStyle {
    fn is_valid(&self) -> Result<bool, String> {
        is_self("style::create", &self.create)?;
        is_self("style::patch", &self.patch)?;
        is_self("style::set_public", &self.set_public)?;
        is_self("style::set_private", &self.set_private)?;
        is_self("style::delete", &self.delete)?;
        is_all("style::get", &self.get)?;
        is_all("style::list", &self.list)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthDelta {
    pub get: Option<String>,
    pub list: Option<String>,
}

impl ValidAuth for AuthDelta {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("delta::get", &self.get)?;
        is_all("delta::list", &self.list)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthFeature {
    pub create: Option<String>,
    pub get: Option<String>,
    pub history: Option<String>
}

impl ValidAuth for AuthFeature {
    fn is_valid(&self) -> Result<bool, String> {
        is_auth("feature::create", &self.create)?;
        is_all("feature::get", &self.get)?;
        is_all("feature::history", &self.history)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthBounds {
    pub list: Option<String>,
    pub get: Option<String>
}

impl ValidAuth for AuthBounds {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("bounds::list", &self.list)?;
        is_all("bounds::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthOSM {
    pub get: Option<String>,
    pub create: Option<String>
}

impl ValidAuth for AuthOSM {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("osm::get", &self.get)?;
        is_auth("osm::create", &self.create)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomAuth {
    pub meta: Option<String>,
    pub mvt: Option<AuthMVT>,
    pub schema: Option<AuthSchema>,
    pub user: Option<AuthUser>,
    pub feature: Option<AuthFeature>,
    pub style: Option<AuthStyle>,
    pub delta: Option<AuthDelta>,
    pub bounds: Option<AuthBounds>,
    pub osm: Option<AuthOSM>
}

impl ValidAuth for CustomAuth {
    fn is_valid(&self) -> Result<bool, String> {
        is_all("meta", &self.meta)?;

        match &self.mvt {
            None => (),
            Some(ref mvt) => { mvt.is_valid()?; }
        };

        match &self.schema {
            None => (),
            Some(ref schema) => { schema.is_valid()?; }
        };
        
        match &self.user {
            None => (),
            Some(ref user) => { user.is_valid()?; }
        };

        match &self.feature {
            None => (),
            Some(ref feature) => { feature.is_valid()?; }
        };

        match &self.style {
            None => (),
            Some(ref style) => { style.is_valid()?; }
        };

        match &self.delta {
            None => (),
            Some(ref delta) => { delta.is_valid()?; }
        };

        match &self.bounds {
            None => (),
            Some(ref bounds) => { bounds.is_valid()?; }
        };

        match &self.osm {
            None => (),
            Some(ref osm) => { osm.is_valid()?; }
        };

        Ok(true)
    }
}


impl CustomAuth {
    pub fn new() -> Self {
        CustomAuth {
            meta: Some(String::from("public")),
            schema: Some(AuthSchema {
                get: Some(String::from("public"))
            }),
            mvt: Some(AuthMVT {
                get: Some(String::from("public")),
                regen: Some(String::from("user")),
                meta: Some(String::from("public"))
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

    pub fn allows_meta(&self, auth: &Auth) -> Result<bool, status::Custom<String>> {


        Ok(true)
    }
}

pub struct Auth {
    pub uid: Option<i64>,
    pub token: Option<String>,
    pub basic: Option<(String, String)>
}

impl Auth {
    pub fn new() -> Self {
        Auth {
            uid: None,
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
    pub fn secure(&mut self, uid: Option<i64>) {
        self.uid = uid;
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
    pub fn validate(mut self, conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Option<i64> {
        if self.basic.is_some() {
            let (username, password) = self.basic.clone().unwrap();

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

                    self.secure(Some(uid));

                    return Some(uid);
                },
                Err(_) => ()
            }
        }

        if self.token.is_some() {
            let token = self.token.clone().unwrap();

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
    
                    self.secure(Some(uid));

                    return Some(uid);
                },
                Err(_) => ()
            }
        };

        None
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Auth, ()> {
        let mut auth = Auth::new();

        match request.cookies().get_private("session") {
            Some(token) => {
                auth.token = Some(String::from(token.value()));

                return Outcome::Success(auth);
            },
            None => ()
        };

        let keys: Vec<_> = request.headers().get("Authorization").collect();

        //Auth Failed - None object returned
        if keys.len() != 1 || keys[0].len() < 7 {
            return Outcome::Success(auth);
        }

        let mut authtype = String::from(keys[0]);
        let auth_str = authtype.split_off(6);

        if authtype != "Basic " {
            return Outcome::Success(auth);
        }

        match base64::decode(&auth_str) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(decoded_str) => {

                    let split = decoded_str.split(":").collect::<Vec<&str>>();

                    if split.len() != 2 { return Outcome::Failure((Status::Unauthorized, ())); }

                    auth.basic = Some((String::from(split[0]), String::from(split[1])));

                    Outcome::Success(auth)
                },
                Err(_) => Outcome::Failure((Status::Unauthorized, ()))
            },
            Err(_) => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
