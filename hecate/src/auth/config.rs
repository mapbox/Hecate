use crate::err::HecateError;
use super::Auth;
pub use crate::user::token::Scope as RW;
use super::AuthAccess;
use hecate_derive::*;

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

pub fn get_kv(scope: &str, key: &str, kv: &serde_json::Value) -> Result<String, HecateError> {
    match kv.get(key) {
        None => Err(HecateError::new(400, format!("{}::{} has no value", scope, key), None)),
        Some(value) => match value.as_str() {
            None => Err(HecateError::new(400, format!("{}::{} value must be string", scope, key), None)),
            Some(value) => Ok(String::from(value))
        }
    }
}

pub trait AuthModule {
    fn default() -> Self;
    fn is_valid(&self) -> Result<bool, String>;
    fn parse(value: Option<&serde_json::Value>) -> Result<Box<Self>, HecateError>;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthWebhooks {
    #[default = "admin"]
    #[valid = "auth"]
    pub get: String,

    #[default = "admin"]
    #[valid = "auth"]
    pub set: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthMeta {
    #[default = "public"]
    #[valid = "all"]
    pub get: String,

    #[default = "admin"]
    #[valid = "auth"]
    pub set: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthClone {
    #[default = "user"]
    #[valid = "all"]
    pub get: String,

    #[default = "user"]
    #[valid = "all"]
    pub query: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthSchema {
    #[default = "public"]
    #[valid = "all"]
    pub get: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthStats {
    #[default = "public"]
    #[valid = "all"]
    pub get: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthAuth {
    #[default = "public"]
    #[valid = "all"]
    pub get: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthMVT {
    #[default = "public"]
    #[valid = "all"]
    pub get: String,

    #[default = "user"]
    #[valid = "all"]
    pub regen: String,

    #[default = "admin"]
    #[valid = "all"]
    pub delete: String,

    #[default = "public"]
    #[valid = "all"]
    pub meta: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthUser {
    #[default = "self"]
    #[valid = "self"]
    pub info: String,

    #[default = "user"]
    #[valid = "all"]
    pub list: String,

    #[default = "public"]
    #[valid = "all"]
    pub create: String,

    #[default = "self"]
    #[valid = "self"]
    pub create_session: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthStyle {
    #[default = "self"]
    #[valid = "self"]
    pub create: String,

    #[default = "self"]
    #[valid = "self"]
    pub patch: String,

    #[default = "self"]
    #[valid = "self"]
    pub set_public: String,

    #[default = "self"]
    #[valid = "self"]
    pub set_private: String,

    #[default = "self"]
    #[valid = "self"]
    pub delete: String,

    #[default = "public"]
    #[valid = "all"]
    pub get: String,

    #[default = "public"]
    #[valid = "all"]
    pub list: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthDelta {
    #[default = "public"]
    #[valid = "all"]
    pub get: String,

    #[default = "public"]
    #[valid = "all"]
    pub list: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthFeature {
    #[default = "disabled"]
    #[valid = "auth"]
    pub force: String,

    #[default = "user"]
    #[valid = "auth"]
    pub create: String,

    #[default = "public"]
    #[valid = "all"]
    pub get: String,

    #[default = "public"]
    #[valid = "all"]
    pub history: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthBounds {
    #[default = "public"]
    #[valid = "all"]
    pub list: String,

    #[default = "admin"]
    #[valid = "all"]
    pub create: String,

    #[default = "admin"]
    #[valid = "all"]
    pub delete: String,

    #[default = "public"]
    #[valid = "all"]
    pub get: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AuthModule)]
pub struct AuthOSM {
    #[default = "public"]
    #[valid = "all"]
    pub get: String,

    #[default = "user"]
    #[valid = "auth"]
    pub create: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CustomAuth {
    pub default: String,
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

impl AuthModule for CustomAuth {
    fn default() -> Self {
        CustomAuth {
            default: String::from("public"),
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

    fn parse(value: Option<&serde_json::Value>) -> Result<Box<Self>, HecateError> {
        match value {
            None => Ok(Box::new(CustomAuth::default())),
            Some(value) => Ok(Box::new(CustomAuth {
                default: get_kv("", "default", value)?,
                server: get_kv("", "server", value)?,
                webhooks: *AuthWebhooks::parse(value.get("webhooks"))?,
                meta: *AuthMeta::parse(value.get("meta"))?,
                stats: *AuthStats::parse(value.get("stats"))?,
                schema: *AuthSchema::parse(value.get("schema"))?,
                auth: *AuthAuth::parse(value.get("auth"))?,
                mvt: *AuthMVT::parse(value.get("mvt"))?,
                user: *AuthUser::parse(value.get("user"))?,
                feature: *AuthFeature::parse(value.get("feature"))?,
                style: *AuthStyle::parse(value.get("style"))?,
                delta: *AuthDelta::parse(value.get("delta"))?,
                bounds: *AuthBounds::parse(value.get("bounds"))?,
                clone: *AuthClone::parse(value.get("clone"))?,
                osm: *AuthOSM::parse(value.get("osm"))?
            }))
        }
    }
}

impl CustomAuth {
    pub fn to_json(&self) -> Result<serde_json::value::Value, HecateError> {
        match serde_json::to_value(&self) {
            Ok(value) => Ok(value),
            Err(err) => Err(HecateError::new(500, String::from("Could not create Auth JSON"), Some(err.to_string())))
        }
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
    // If an account is disabled, all endpoints fail,
    // regardless of whether they are public or user/admin
    if auth.access == AuthAccess::Disabled {
        return Err(not_authed());
    }

    match required {
        None => Err(not_authed()),
        Some(req) => match req.as_ref() {
            "public" => Ok(true),
            "admin" => {
                if auth.uid.is_some() && auth.access == AuthAccess::Admin {
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
