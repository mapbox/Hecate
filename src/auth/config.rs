use crate::err::HecateError;
use super::Auth;
pub use crate::user::token::Scope as RW;

pub fn not_authed() -> HecateError {
    HecateError::new(401, String::from("You must be logged in to access this resource"), None)
}

#[derive(Clone)]
pub struct AuthContainer(pub CustomAuth);

///
/// Allows a category to be null, public, admin, or user
///
/// This category makes up the majority of endpoints in hecate and is the most
/// flexible
///
fn is_all(scope_type: &str, scope: &String) -> Result<bool, String> {
    match scope.as_ref() {
        "public" => Ok(true),
        "admin" => Ok(true),
        "user" => Ok(true),
        "disabled" => Ok(true),
        _ => Err(format!("Auth Config Error: '{}' must be one of 'public', 'admin', 'user', or 'disabled'", scope_type)),
    }
}

///
/// Allows a category to be null, self, or admin
///
/// This category is used for CRUD operations against data for a specfic user,
/// not only must the user be logged in but the user can only update their own
/// data
///
fn is_self(scope_type: &str, scope: &String) -> Result<bool, String> {
    match scope.as_ref() {
        "self" => Ok(true),
        "admin" => Ok(true),
        "disabled" => Ok(true),
        _ => Err(format!("Auth Config Error: '{}' must be one of 'self', 'admin', or 'disabled'", scope_type)),
    }
}

///
/// Allows a category to be null, user, or admin
///
/// This category is used primarily for feature operations. The user must be
/// logged in but can make changes to any feature, including features created
/// by another user
///
fn is_auth(scope_type: &str, scope: &String) -> Result<bool, String> {
    match scope.as_ref() {
        "user" => Ok(true),
        "admin" => Ok(true),
        "disabled" => Ok(true),
        _ => Err(format!("Auth Config Error: '{}' must be one of 'user', 'admin', or 'disabled'", scope_type)),
    }
}

pub trait SubAuth {
    fn default() -> Self;
    fn parse(value: &Option<serde_json::Value>) -> Self;
    fn is_valid(&self) -> Result<bool, String>;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthWebhooks {
    pub get: String,
    pub set: String
}

impl SubAuth for AuthWebhooks {
    fn default() -> Self {
        AuthWebhooks {
            get: String::from("admin"),
            set: String::from("admin")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthWebhooks {
            get: String::from("admin"),
            set: String::from("admin")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_auth("webhooks::get", &self.get)?;
        is_auth("webhooks::set", &self.set)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthMeta {
    pub get: String,
    pub set: String
}

impl SubAuth for AuthMeta {
    fn default() -> Self {
        AuthMeta {
            get: String::from("public"),
            set: String::from("admin")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthMeta {
            get: String::from("public"),
            set: String::from("admin")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("meta::get", &self.get)?;
        is_auth("meta::set", &self.set)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthClone {
    pub get: String,
    pub query: String
}

impl SubAuth for AuthClone {
    fn default() -> Self {
        AuthClone {
            get: String::from("user"),
            query: String::from("user")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthClone {
            get: String::from("user"),
            query: String::from("user")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("clone::get", &self.get)?;
        is_all("clone::query", &self.query)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthSchema {
    pub get: String
}

impl SubAuth for AuthSchema {
    fn default() -> Self {
        AuthSchema {
            get: String::from("public")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthSchema {
            get: String::from("public")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("schema::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthStats {
    pub get: String
}

impl SubAuth for AuthStats {
    fn default() -> Self {
        AuthStats {
            get: String::from("public"),
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthStats {
            get: String::from("public"),
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("stats::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthAuth {
    pub get: String
}

impl SubAuth for AuthAuth {
    fn default() -> Self {
        AuthAuth {
            get: String::from("public")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthAuth {
            get: String::from("public")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("auth::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthMVT {
    pub get: String,
    pub delete: String,
    pub regen: String,
    pub meta: String
}

impl SubAuth for AuthMVT {
    fn default() -> Self {
        AuthMVT {
            get: String::from("public"),
            delete: String::from("admin"),
            regen: String::from("user"),
            meta: String::from("public")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthMVT {
            get: String::from("public"),
            delete: String::from("admin"),
            regen: String::from("user"),
            meta: String::from("public")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("mvt::get", &self.get)?;
        is_all("mvt::regen", &self.regen)?;
        is_all("mvt::delete", &self.regen)?;
        is_all("mvt::meta", &self.meta)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthUser {
    pub info: String,
    pub list: String,
    pub create: String,
    pub create_session: String
}

impl SubAuth for AuthUser {
    fn default() -> Self {
        AuthUser {
            info: String::from("self"),
            list: String::from("user"),
            create: String::from("public"),
            create_session: String::from("self")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthUser {
            info: String::from("self"),
            list: String::from("user"),
            create: String::from("public"),
            create_session: String::from("self")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("user::create", &self.create)?;
        is_all("user::list", &self.list)?;

        is_self("user::create_session", &self.create_session)?;
        is_self("user::info", &self.info)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthStyle {
    pub create: String,
    pub patch: String,
    pub set_public: String,
    pub set_private: String,
    pub delete: String,
    pub get: String,
    pub list: String
}

impl SubAuth for AuthStyle {
    fn default() -> Self {
        AuthStyle {
            create: String::from("self"),
            patch: String::from("self"),
            set_public: String::from("self"),
            set_private: String::from("self"),
            delete: String::from("self"),
            get: String::from("public"),
            list: String::from("public")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthStyle {
            create: String::from("self"),
            patch: String::from("self"),
            set_public: String::from("self"),
            set_private: String::from("self"),
            delete: String::from("self"),
            get: String::from("public"),
            list: String::from("public")
        }
    }

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthDelta {
    pub get: String,
    pub list: String,
}

impl SubAuth for AuthDelta {
    fn default() -> Self {
        AuthDelta {
            get: String::from("public"),
            list: String::from("public")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthDelta {
            get: String::from("public"),
            list: String::from("public")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("delta::get", &self.get)?;
        is_all("delta::list", &self.list)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthFeature {
    pub force: String,
    pub create: String,
    pub get: String,
    pub history: String
}

impl SubAuth for AuthFeature {
    fn default() -> Self {
        AuthFeature {
            force: String::from("none"),
            create: String::from("user"),
            get: String::from("public"),
            history: String::from("public")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthFeature {
            force: String::from("none"),
            create: String::from("user"),
            get: String::from("public"),
            history: String::from("public")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_auth("feature::create", &self.create)?;
        is_auth("feature::force", &self.force)?;
        is_all("feature::get", &self.get)?;
        is_all("feature::history", &self.history)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthBounds {
    pub list: String,
    pub create: String,
    pub delete: String,
    pub get: String
}

impl SubAuth for AuthBounds {
    fn default() -> Self {
        AuthBounds {
            list: String::from("public"),
            create: String::from("admin"),
            delete: String::from("admin"),
            get: String::from("public")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthBounds {
            list: String::from("public"),
            create: String::from("admin"),
            delete: String::from("admin"),
            get: String::from("public")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("bounds::list", &self.list)?;
        is_all("bounds::create", &self.create)?;
        is_all("bounds::delete", &self.create)?;
        is_all("bounds::get", &self.get)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AuthOSM {
    pub get: String,
    pub create: String
}

impl SubAuth for AuthOSM {
    fn default() -> Self {
        AuthOSM {
            get: String::from("public"),
            create: String::from("user")
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        AuthOSM {
            get: String::from("public"),
            create: String::from("user")
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("osm::get", &self.get)?;
        is_auth("osm::create", &self.create)?;

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CustomAuth {
    pub default: Option<String>,
    pub server: String,
    pub meta: AuthMeta,
    pub webhooks: AuthWebhooks,
    pub stats: AuthStats,
    pub mvt: AuthMVT,
    pub schema: AuthSchema,
    pub auth: AuthAuth,
    pub user: AuthUser,
    pub feature: AuthFeature,
    pub style: AuthStyle,
    pub delta: AuthDelta,
    pub bounds: AuthBounds,
    pub clone: AuthClone,
    pub osm: AuthOSM
}

impl SubAuth for CustomAuth {
    fn default() -> Self {
        CustomAuth {
            default: Some(String::from("public")),
            server: String::from("public"),
            webhooks: AuthWebhooks::default(),
            meta: AuthMeta::default(),
            stats: AuthStats::default(),
            schema: AuthSchema::default(),
            auth: AuthAuth::default(),
            mvt: AuthMVT::default(),
            user: AuthUser::default(),
            feature: AuthFeature::default(),
            style: AuthStyle::default(),
            delta: AuthDelta::default(),
            bounds: AuthBounds::default(),
            clone: AuthClone::default(),
            osm: AuthOSM::default()
        }
    }

    fn parse(value: &Option<serde_json::Value>) -> Self {
        CustomAuth {
            default: Some(String::from("public")),
            server: String::from("public"),
            webhooks: AuthWebhooks::parse(&value),
            meta: AuthMeta::parse(&value),
            stats: AuthStats::parse(&value),
            schema: AuthSchema::parse(&value),
            auth: AuthAuth::parse(&value),
            mvt: AuthMVT::parse(&value),
            user: AuthUser::parse(&value),
            feature: AuthFeature::parse(&value),
            style: AuthStyle::parse(&value),
            delta: AuthDelta::parse(&value),
            bounds: AuthBounds::parse(&value),
            clone: AuthClone::parse(&value),
            osm: AuthOSM::parse(&value)
        }

    }

    fn is_valid(&self) -> Result<bool, String> {
        is_all("server", &self.server)?;

        &self.meta.is_valid()?;
        &self.mvt.is_valid()?;
        &self.stats.is_valid()?;
        &self.clone.is_valid()?;
        &self.schema.is_valid()?;
        &self.user.is_valid()?;
        &self.feature.is_valid()?;
        &self.style.is_valid()?;
        &self.delta.is_valid()?;
        &self.bounds.is_valid()?;
        &self.osm.is_valid()?;

        Ok(true)
    }
}

impl CustomAuth {
    pub fn to_json(&self) -> serde_json::value::Value {
        let json_auth = serde_json::from_str(serde_json::to_string(&self).unwrap().as_str()).unwrap();

        json_auth
    }


    pub fn is_admin(&self, auth: &Auth) -> Result<bool, HecateError> {
        auth_met(&Some(String::from("admin")), auth)
    }
}

pub fn rw_met(rw: RW, auth: &Auth) -> Result<(), HecateError> {
    if rw == RW::Full && auth.scope == RW::Read {
        return Err(not_authed());
    }

    return Ok(());
}

///
/// Determines whether the current auth state meets or exceeds the
/// requirements of an endpoint
///
fn auth_met(required: &Option<String>, auth: &Auth) -> Result<bool, HecateError> {
    match required {
        None => Err(not_authed()),
        Some(req) => match req.as_ref() {
            "public" => Ok(true),
            "admin" => {
                if auth.uid.is_none() || auth.access.is_none() {
                    return Err(not_authed());
                } else if auth.access == Some(String::from("admin")) {
                    return Ok(true);
                } else {
                    return Err(not_authed());
                }
            },
            "user" => {
                if auth.uid.is_some() {
                    return Ok(true);
                } else {
                    return Err(not_authed());
                }
            },
            "self" => {
                //Note: This ensures the user is validated,
                //it is up to the parent caller to ensure
                //the UID of 'self' matches the requested resource

                if auth.uid.is_some() {
                    return Ok(true);
                } else {
                    return Err(not_authed());
                }
            },
            _ => Err(not_authed())
        }
    }
}
