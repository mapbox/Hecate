#![feature(proc_macro_hygiene, decl_macro, plugin, custom_derive, custom_attribute)]

static VERSION: &'static str = "0.49.3";

#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate postgis;
extern crate protobuf;
extern crate rand;
extern crate valico;
extern crate rocket_contrib;
extern crate geojson;
extern crate env_logger;
extern crate chrono;

pub mod meta;
pub mod stats;
pub mod delta;
pub mod mvt;
pub mod feature;
pub mod bounds;
pub mod clone;
pub mod stream;
pub mod style;
pub mod xml;
pub mod user;
pub mod auth;

use auth::ValidAuth;

//Postgres Connection Pooling
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use mvt::Encode;

use rand::prelude::*;

use std::io::{Cursor};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use rocket::http::Status as HTTPStatus;
use rocket::config::{Config, Environment, LoggingLevel, Limits};
use rocket::http::{Cookie, Cookies};
use rocket::{State};
use rocket::response::{Response, status, Stream, NamedFile};
use rocket::request::Form;
use geojson::GeoJson;
use rocket_contrib::json::Json;

pub fn start(
    database: String,
    database_read: Vec<String>,
    port: Option<u16>,
    workers: Option<u16>,
    schema: Option<serde_json::value::Value>,
    auth: Option<auth::CustomAuth>
) {
    env_logger::init();

    let auth_rules: auth::CustomAuth = match auth {
        None => auth::CustomAuth::new(),
        Some(auth) => {
            match auth.is_valid() {
                Err(err_msg) => { panic!(err_msg); },
                Ok(_) => ()
            };

            auth
        }
    };

    let db_read: DbRead = DbRead::new(Some(database_read.iter().map(|db| init_pool(&db)).collect()));

    let limits = Limits::new()
        .limit("json", 20971520)
        .limit("forms", 131072);

    let config = Config::build(Environment::Production)
        .address("0.0.0.0")
        .log_level(LoggingLevel::Debug)
        .port(port.unwrap_or(8000))
        .limits(limits)
        .workers(workers.unwrap_or(12))
        .unwrap();

    rocket::custom(config)
        .manage(DbReadWrite::new(init_pool(&database)))
        .manage(db_read)
        .manage(schema)
        .manage(auth_rules)
        .mount("/", routes![
            index
        ])
//      .mount("/admin", routes![
//          staticsrv,
//          staticsrvredirect
//      ])
//      .mount("/api", routes![
//          server,
//          meta_list,
//          meta_get,
//          meta_delete,
//          meta_set,
//          schema_get,
//          auth_get,
//          stats_get,
//          mvt_get,
//          mvt_meta,
//          mvt_wipe,
//          mvt_regen,
//          user_self,
//          user_info,
//          user_create,
//          user_set_admin,
//          user_delete_admin,
//          user_list,
//          user_filter,
//          user_create_session,
//          user_delete_session,
//          style_create,
//          style_patch,
//          style_public,
//          style_private,
//          style_delete,
//          style_get,
//          style_list_public,
//          style_list_user,
//          delta,
//          delta_list,
//          delta_list_params,
//          feature_action,
//          features_action,
//          feature_get,
//          feature_query,
//          feature_get_history,
//          features_get,
//          bounds_list,
//          bounds_filter,
//          bounds_stats,
//          bounds_get,
//          bounds_set,
//          bounds_delete,
//          clone_get,
//          clone_query,
//          xml_capabilities,
//          xml_06capabilities,
//          xml_user,
//          xml_map,
//          xml_changeset_create,
//          xml_changeset_modify,
//          xml_changeset_upload,
//          xml_changeset_close
//      ])
        .register(catchers![
           not_authorized,
           not_found,
        ]).launch();
}


pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

fn init_pool(database: &str) -> r2d2::Pool<r2d2_postgres::PostgresConnectionManager> {
    //Create Postgres Connection Pool
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(format!("postgres://{}", database), TlsMode::None).unwrap();
    match r2d2::Pool::builder().max_size(15).build(manager) {
        Ok(pool) => pool,
        Err(_) => { panic!("Failed to connect to database"); }
    }
}

pub struct DbRead(pub Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager>>>);
impl DbRead {
    fn new(database: Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager>>>) -> Self {
        DbRead(database)
    }

    fn get(&self) -> Result<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, status::Custom<Json<serde_json::Value>>> {
        match self.0 {
            None => Err(status::Custom(HTTPStatus::ServiceUnavailable, Json(json!({
                "code": 503,
                "status": "Service Unavailable",
                "reason": "No Database Read Connection"
            })))),
            Some(ref db_read) => {
                let mut rng = thread_rng();
                let db_read_it = rng.gen_range(0, db_read.len());

                match db_read.get(db_read_it).unwrap().get() {
                    Ok(conn) => Ok(conn),
                    Err(_) => Err(status::Custom(HTTPStatus::ServiceUnavailable, Json(json!({
                        "code": 503,
                        "status": "Service Unavailable",
                        "reason": "Could not connect to database"
                    }))))
                }
            }
        }
    }
}

pub struct DbReadWrite(pub r2d2::Pool<r2d2_postgres::PostgresConnectionManager>); //Read & Write DB Connection
impl DbReadWrite {
    fn new(database: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>) -> Self {
        DbReadWrite(database)
    }

    fn get(&self) -> Result<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, status::Custom<Json<serde_json::Value>>> {
        match self.0.get() {
            Ok(conn) => Ok(conn),
            Err(_) => Err(status::Custom(HTTPStatus::ServiceUnavailable, Json(json!({
                "code": 503,
                "status": "Service Unavailable",
                "reason": "Could not connect to database"
            }))))
        }
    }
}

#[derive(FromForm)]
struct Filter {
    filter: String,
    limit: Option<i32>
}

#[catch(401)]
fn not_authorized() -> Json<serde_json::Value> {
    Json(json!({
        "code": 401,
        "status": "Not Authorized",
        "reason": "You must be logged in to access this resource"
    }))
}

#[catch(404)]
fn not_found() -> Json<serde_json::Value> {
    Json(json!({
        "code": 404,
        "status": "Not Found",
        "reason": "Resource was not found."
    }))
}


#[get("/")]
fn index() -> &'static str { "Hello World!" }

/*
#[get("/")]
fn server(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    auth_rules.allows_server(&mut auth, &conn.get()?)?;

    Ok(Json(json!({
        "version": VERSION
    })))
}

#[get("/meta")]
fn meta_list(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_meta_list(&mut auth, &conn)?;

    match meta::list(&conn) {
        Ok(list) => {
            Ok(Json(json!(list)))
        },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/meta/<key>")]
fn meta_get(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>, key: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_meta_get(&mut auth, &conn)?;

    match meta::get(&conn, &key) {
        Ok(list) => {
            Ok(Json(json!(list)))
        },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[delete("/meta/<key>")]
fn meta_delete(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>, key: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_meta_set(&mut auth, &conn)?;

    match meta::delete(&conn, &key) {
        Ok(_) => Ok(Json(json!(true))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[post("/meta/<key>", format="application/json", data="<body>")]
fn meta_set(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>, key: String, body: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_meta_set(&mut auth, &conn)?;

    match meta::set(&conn, &key, &body) {
        Ok(_) => Ok(Json(json!(true))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/")]
fn staticsrvredirect() -> rocket::response::Redirect {
    rocket::response::Redirect::to("/admin/index.html")
}

#[get("/<file..>")]
fn staticsrv(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("web/dist/").join(file)).ok()
}

#[get("/tiles/<z>/<x>/<y>")]
fn mvt_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Response<'static>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_mvt_get(&mut auth, &conn)?;

    if z > 14 { return Err(status::Custom(HTTPStatus::NotFound, Json(json!("Tile Not Found")))); }

    let tile = match mvt::get(&conn, z, x, y, false) {
        Ok(tile) => tile,
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string())))); }
    };

    let mut c = Cursor::new(Vec::new());
    match tile.to_writer(&mut c) {
        Ok(_) => (),
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string())))); }
    }

    let mut mvt_response = Response::new();
    mvt_response.set_status(HTTPStatus::Ok);
    mvt_response.set_sized_body(c);
    mvt_response.set_raw_header("Content-Type", "application/x-protobuf");
    Ok(mvt_response)
}

#[get("/tiles/<z>/<x>/<y>/meta")]
fn mvt_meta(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_mvt_meta(&mut auth, &conn)?;

    if z > 14 { return Err(status::Custom(HTTPStatus::NotFound, Json(json!("Tile Not Found")))); }

    match mvt::meta(&conn, z, x, y) {
        Ok(tile) => Ok(Json(tile)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}


#[delete("/tiles")]
fn mvt_wipe(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_mvt_delete(&mut auth, &conn)?;

    match mvt::wipe(&conn) {
        Ok(response) => Ok(Json(response)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/tiles/<z>/<x>/<y>/regen")]
fn mvt_regen(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Response<'static>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_mvt_regen(&mut auth, &conn)?;

    if z > 14 { return Err(status::Custom(HTTPStatus::NotFound, Json(json!("Tile Not Found")))); }

    let tile = match mvt::get(&conn, z, x, y, true) {
        Ok(tile) => tile,
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string())))); }
    };

    let mut c = Cursor::new(Vec::new());
    match tile.to_writer(&mut c) {
        Ok(_) => (),
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string())))); }
    }

    let mut mvt_response = Response::new();
    mvt_response.set_status(HTTPStatus::Ok);
    mvt_response.set_sized_body(c);
    mvt_response.set_raw_header("Content-Type", "application/x-protobuf");
    Ok(mvt_response)
}
*/

#[derive(FromForm)]
struct User {
    username: String,
    password: String,
    email: String
}

#[derive(FromForm)]
struct Map {
    bbox: String
}

#[get("/user/create?<user..>")]
fn user_create(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, user: Form<User>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_user_create(&mut auth, &conn)?;

    match user::create(&conn, &user.username, &user.password, &user.email) {
        Ok(_) => Ok(Json(json!(true))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/users")]
fn user_list(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_user_list(&mut auth, &conn)?;

    match user::list(&conn) {
        Ok(users) => Ok(Json(json!(users))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/users?<filter..>")]
fn user_filter(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, filter: Form<Filter>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_user_list(&mut auth, &conn)?;

    match user::filter(&conn, &filter.filter) {
        Ok(users) => Ok(Json(json!(users))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/user/<id>")]
fn user_info(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.is_admin(&mut auth, &conn)?;

    match user::info(&conn, &id) {
        Ok(info) => { Ok(Json(info)) },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[put("/user/<id>/admin")]
fn user_set_admin(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.is_admin(&mut auth, &conn)?;

    match user::set_admin(&conn, &id) {
        Ok(info) => { Ok(Json(json!(info))) },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[delete("/user/<id>/admin")]
fn user_delete_admin(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.is_admin(&mut auth, &conn)?;

    match user::delete_admin(&conn, &id) {
        Ok(info) => { Ok(Json(json!(info))) },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/user/info")]
fn user_self(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_user_info(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    match user::info(&conn, &uid) {
        Ok(info) => { Ok(Json(info)) },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/user/session")]
fn user_create_session(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, mut cookies: Cookies) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_user_create_session(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    match user::create_token(&conn, &uid) {
        Ok(token) => {
            cookies.add_private(Cookie::new("session", token));
            Ok(Json(json!(uid)))
        },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[delete("/user/session")]
fn user_delete_session(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, mut cookies: Cookies) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_user_create_session(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    match cookies.get_private("session") {
        Some(session) => {
            let token = String::from(session.value());

            match user::destroy_token(&conn, &uid, &token) {
                _ => {
                    cookies.remove_private(session); 
                    Ok(Json(json!(true)))
                }
            }
        },
        None => Ok(Json(json!(true)))
    }
}

#[post("/style", format="application/json", data="<style>")]
fn style_create(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, style: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    
    auth_rules.allows_style_create(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    match style::create(&conn, &uid, &style) {
        Ok(style_id) => Ok(Json(json!(style_id))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[post("/style/<id>/public")]
fn style_public(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_style_set_public(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    match style::access(&conn, &uid, &id, true) {
        Ok(updated) => Ok(Json(json!(updated))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[post("/style/<id>/private")]
fn style_private(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_style_set_private(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    match style::access(&conn, &uid, &id, false) {
        Ok(updated) => Ok(Json(json!(updated))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[patch("/style/<id>", format="application/json", data="<style>")]
fn style_patch(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64, style: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_style_patch(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    match style::update(&conn, &uid, &id, &style) {
        Ok(updated) => Ok(Json(json!(updated))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[delete("/style/<id>")]
fn style_delete(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_style_delete(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    match style::delete(&conn, &uid, &id) {
        Ok(created) => Ok(Json(json!(created))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}


#[get("/style/<id>")]
fn style_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_style_get(&mut auth, &conn)?;

    match style::get(&conn, &auth.uid, &id) {
        Ok(style) => Ok(Json(json!(style))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/styles")]
fn style_list_public(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_style_list(&mut auth, &conn)?;

    match style::list_public(&conn) {
        Ok(styles) => Ok(Json(json!(styles))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/styles/<user>")]
fn style_list_user(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, user: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_style_list(&mut auth, &conn)?;

    match auth.uid {
        Some(uid) => {
            if uid == user {
                match style::list_user(&conn, &user) {
                    Ok(styles) => Ok(Json(json!(styles))),
                    Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
                }
            } else {
                match style::list_user_public(&conn, &user) {
                    Ok(styles) => Ok(Json(json!(styles))),
                    Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
                }
            }
        },
        _ => {
            match style::list_user_public(&conn, &user) {
                Ok(styles) => Ok(Json(json!(styles))),
                Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
            }
        }
    }
}

#[get("/deltas")]
fn delta_list(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) ->  Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_delta_list(&mut auth, &conn)?;

    match delta::list_by_offset(&conn, None, None) {
        Ok(deltas) => Ok(Json(deltas)),
        Err(err) => Err(status::Custom(HTTPStatus::InternalServerError, Json(json!(err.to_string()))))
    }
}

#[derive(FromForm)]
struct DeltaList {
    offset: Option<i64>,
    limit: Option<i64>,
    start: Option<String>,
    end: Option<String>
}

#[get("/deltas?<opts..>")]
fn delta_list_params(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, opts: Form<DeltaList>) ->  Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_delta_list(&mut auth, &conn)?;

    if opts.offset.is_some() && (opts.start.is_some() || opts.end.is_some()) {
        return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Offset cannot be used with start or end"))));
    }

    if opts.start.is_some() || opts.end.is_some() {
        let start: Option<chrono::NaiveDateTime> = match &opts.start {
            None => None,
            Some(start) => {
                match start.parse() {
                    Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Invalid start timestamp")))); },
                    Ok(start) => Some(start)
                }
            }
        };

        let end: Option<chrono::NaiveDateTime> = match &opts.end {
            None => None,
            Some(end) => {
                match end.parse() {
                    Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Invalid end timestamp")))); },
                    Ok(end) => Some(end)
                }
            }
        };

        match delta::list_by_date(&conn, start, end, opts.limit) {
            Ok(deltas) => {
                return Ok(Json(deltas));
            },
            Err(err) => {
                return Err(status::Custom(HTTPStatus::InternalServerError, Json(json!(err.to_string()))));
            }
        }
    } else if opts.offset.is_some() || opts.limit.is_some() {
        match delta::list_by_offset(&conn, opts.offset, opts.limit) {
            Ok(deltas) => {
                return Ok(Json(deltas));
            },
            Err(err) => {
                return Err(status::Custom(HTTPStatus::InternalServerError, Json(json!(err.to_string()))));
            }
        }
    }
    Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Query Param Error"))))
}

#[get("/delta/<id>")]
fn delta(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) ->  Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_delta_get(&mut auth, &conn)?;

    match delta::get_json(&conn, &id) {
        Ok(delta) => Ok(Json(delta)),
        Err(err) => Err(status::Custom(HTTPStatus::InternalServerError, Json(json!(err.to_string()))))
    }
}

#[get("/data/bounds")]
fn bounds_list(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_list(&mut auth, &conn)?;

    match bounds::list(&conn, None) {
        Ok(bounds) => Ok(Json(json!(bounds))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/data/bounds?<filter..>")]
fn bounds_filter(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, filter: Form<Filter>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_list(&mut auth, &conn)?;

    match bounds::search(&conn, &filter.filter, &filter.limit) {
        Ok(bounds) => Ok(Json(json!(bounds))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/data/bounds/<bounds>")]
fn bounds_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String) -> Result<Stream<stream::PGStream>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_list(&mut auth, &conn)?;

    match bounds::get(conn, bounds) {
        Ok(bs) => Ok(Stream::from(bs)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[post("/data/bounds/<bounds>", format="application/json", data="<body>")]
fn bounds_set(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String, body: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_create(&mut auth, &conn)?;

    let geom: serde_json::Value = match serde_json::from_str(&*body) {
        Ok(geom) => geom,
        Err(_) => {
            return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Invalid Feature GeoJSON"))));
        }
    };

    match bounds::set(&conn, &bounds, &geom) {
        Ok(_) => Ok(Json(json!(true))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[delete("/data/bounds/<bounds>")]
fn bounds_delete(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_delete(&mut auth, &conn)?;

    match bounds::delete(&conn, &bounds) {
        Ok(_) => Ok(Json(json!(true))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[get("/data/bounds/<bounds>/stats")]
fn bounds_stats(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_stats_bounds(&mut auth, &conn)?;

    match bounds::stats_json(conn, bounds) {
        Ok(stats) => Ok(Json(stats)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}

#[derive(FromForm)]
struct CloneQuery {
    query: String,
    limit: Option<i64>
}

#[get("/data/query?<cquery..>")]
fn clone_query(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, cquery: Form<CloneQuery>) -> Result<Stream<stream::PGStream>, status::Custom<Json<serde_json::Value>>> {
    auth_rules.allows_clone_query(&mut auth, &conn.get()?)?;

    match clone::query(read_conn.get()?, &cquery.query, &cquery.limit) {
        Ok(clone) => Ok(Stream::from(clone)),
        Err(err) => Err(err)
    }
}

#[get("/data/clone")]
fn clone_get(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Stream<stream::PGStream>, status::Custom<Json<serde_json::Value>>> {
    auth_rules.allows_clone_get(&mut auth, &conn.get()?)?;

    match clone::get(read_conn.get()?) {
        Ok(clone) => Ok(Stream::from(clone)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(err.as_json())))
    }
}

#[get("/data/features?<map..>")]
fn features_get(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, map: Form<Map>) -> Result<Stream<stream::PGStream>, status::Custom<Json<serde_json::Value>>> {
    auth_rules.allows_feature_get(&mut auth, &conn.get()?)?;

    let bbox: Vec<f64> = map.bbox.split(',').map(|s| s.parse().unwrap()).collect();
    match feature::get_bbox_stream(read_conn.get()?, bbox) {
        Ok(features) => Ok(Stream::from(features)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(err.as_json())))
    }
}

#[get("/schema")]
fn schema_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, schema: State<Option<serde_json::value::Value>>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_schema_get(&mut auth, &conn)?;

    match schema.inner().clone() {
        Some(s) => Ok(Json(json!(s.clone()))),
        None => Err(status::Custom(HTTPStatus::NotFound, Json(json!("No Schema Validation Enforced"))))
    }
}

#[get("/auth")]
fn auth_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_auth_get(&mut auth, &conn)?;

    Ok(Json(auth_rules.to_json()))
}

#[get("/data/stats")]
fn stats_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_stats_get(&mut auth, &conn)?;

    match stats::get_json(&conn) {
        Ok(stats) => Ok(Json(stats)),
        Err(err) => Err(status::Custom(HTTPStatus::InternalServerError, Json(json!(err.to_string()))))
    }
}

/*
#[post("/data/features", format="application/json", data="<body>")]
fn features_action(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, schema: State<Option<serde_json::value::Value>>, body: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_feature_create(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    let mut fc = match body.parse::<GeoJson>() {
        Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Body must be valid GeoJSON Feature")))); },
        Ok(geo) => match geo {
            GeoJson::FeatureCollection(fc) => fc,
            _ => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Body must be valid GeoJSON FeatureCollection")))); }
        }
    };

    let delta_message = match fc.foreign_members {
        None => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("FeatureCollection Must have message property for delta")))); }
        Some(ref members) => match members.get("message") {
            Some(message) => match message.as_str() {
                Some(message) => String::from(message),
                None => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("FeatureCollection Must have message property for delta")))); }
            },
            None => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("FeatureCollection Must have message property for delta")))); }
        }
    };

    let trans = conn.transaction().unwrap();

    let mut map: HashMap<String, Option<String>> = HashMap::new();
    map.insert(String::from("message"), Some(delta_message));

    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, Json(json!("Could not create delta")))); }
    };

    for feat in &mut fc.features {
        match feature::is_force(&feat) {
            Err(err) => {
                return Err(status::Custom(HTTPStatus::ExpectationFailed, Json(err.as_json())));
            },
            Ok(force) => {
                if force {
                    auth_rules.allows_feature_force(&mut auth, &conn)?;
                }
            }
        };

        match feature::action(&trans, &schema.inner(), &feat, &None) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(status::Custom(HTTPStatus::ExpectationFailed, Json(err.as_json())));
            },
            Ok(res) => {
                if res.new != None {
                    feat.id = Some(Json(json!(res.new)))
                }
            }
        }
    }

    if delta::modify(&delta_id, &trans, &fc, &uid).is_err() {
        trans.set_rollback();
        trans.finish().unwrap();
        return Err(status::Custom(HTTPStatus::InternalServerError, Json(json!("Could not create delta"))));
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Json(json!(true)))
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Err(status::Custom(HTTPStatus::InternalServerError, Json(json!(err.to_string()))))
        }
    }
}
*/

#[get("/0.6/map?<map..>")]
fn xml_map(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, map: Form<Map>) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap(); 

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let query: Vec<f64> = map.bbox.split(',').map(|s| s.parse().unwrap()).collect();

    let fc = match feature::get_bbox(&conn, query) {
        Ok(features) => features,
        Err(err) => { return Err(status::Custom(HTTPStatus::ExpectationFailed, err.as_json().to_string())) }
    };

    let xml_str = match xml::from_features(&fc) {
        Ok(xml_str) => xml_str,
        Err(err) => { return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string())) }
    };

    Ok(xml_str)
}

#[put("/0.6/changeset/create", data="<body>")]
fn xml_changeset_create(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, body: String) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let uid = auth.uid.unwrap();

    let map = match xml::to_delta(&body) {
        Ok(map) => map,
        Err(err) => { return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string())); }
    };

    let trans = conn.transaction().unwrap();

    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
        }
    };

    trans.commit().unwrap();

    Ok(delta_id.to_string())
}

#[put("/0.6/changeset/<id>/close")]
fn xml_changeset_close(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, id: i64) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    Ok(id.to_string())
}

#[put("/0.6/changeset/<delta_id>", data="<body>")]
fn xml_changeset_modify(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, delta_id: i64, body: String) -> Result<Response<'static>, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let uid = auth.uid.unwrap();

    let trans = conn.transaction().unwrap();

    match delta::is_open(&delta_id, &trans) {
        Ok(true) => (),
        _ => {
            trans.set_rollback();
            trans.finish().unwrap();

            let mut conflict_response = Response::new();
            conflict_response.set_status(HTTPStatus::Conflict);
            conflict_response.set_sized_body(Cursor::new(format!("The changeset {} was closed at previously", &delta_id)));
            conflict_response.set_raw_header("Error", format!("The changeset {} was closed at previously", &delta_id));
            return Ok(conflict_response);
        }
    }

    let map = match xml::to_delta(&body) {
        Ok(map) => map,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
        }
    };

    let delta_id = match delta::modify_props(&delta_id, &trans, &map, &uid) {
        Ok(id) => id,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
        }
    };

    trans.commit().unwrap();

    Err(status::Custom(HTTPStatus::Ok, delta_id.to_string()))
}

#[post("/0.6/changeset/<delta_id>/upload", data="<body>")]
fn xml_changeset_upload(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, schema: State<Option<serde_json::value::Value>>, delta_id: i64, body: String) -> Result<Response<'static>, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let uid = auth.uid.unwrap();

    let trans = conn.transaction().unwrap();

    match delta::is_open(&delta_id, &trans) {
        Ok(true) => (),
        _ => {
            trans.set_rollback();
            trans.finish().unwrap();

            let mut conflict_response = Response::new();
            conflict_response.set_status(HTTPStatus::Conflict);
            conflict_response.set_sized_body(Cursor::new(format!("The changeset {} was closed at previously", &delta_id)));
            conflict_response.set_raw_header("Error", format!("The changeset {} was closed at previously", &delta_id));
            return Ok(conflict_response);
        }
    }

    let (mut fc, tree) = match xml::to_features(&body) {
        Ok(fctree) => fctree,
        Err(err) => { return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string())); }
    };

    let mut ids: HashMap<i64, feature::Response> = HashMap::new();

    for feat in &mut fc.features {
        match feature::get_action(&feat) {
            Ok(action) => {
                if action == feature::Action::Create {
                    feature::del_version(feat);
                }
            },
            _ => ()
        }

        let feat_res = match feature::action(&trans, &schema.inner(), &feat, &Some(delta_id)) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(status::Custom(HTTPStatus::ExpectationFailed, err.as_json().to_string()));
            },
            Ok(feat_res) => {
                if feat_res.old.unwrap_or(0) < 0 {
                    feat.id = Some(json!(feat_res.new));
                }

                feat_res
            }
        };

        ids.insert(feat_res.old.unwrap(), feat_res);
    }

    let diffres = match xml::to_diffresult(ids, tree) {
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not format diffResult XML")));
        },
        Ok(diffres) => diffres
    };

    match delta::modify(&delta_id, &trans, &fc, &uid) {
        Ok (_) => (),
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not create delta")));
        }
    }

    match delta::finalize(&delta_id, &trans) {
        Ok (_) => {
            trans.commit().unwrap();
            Err(status::Custom(HTTPStatus::Ok, diffres))
        },
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not close delta")))
        }
    }
}

#[get("/capabilities")]
fn xml_capabilities(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    Ok(String::from("
        <osm version=\"0.6\" generator=\"Hecate Server\">
            <api>
                <version minimum=\"0.6\" maximum=\"0.6\"/>
                <area maximum=\"0.25\"/>
                <waynodes maximum=\"2000\"/>
                <changesets maximum_elements=\"10000\"/>
                <timeout seconds=\"300\"/>
                <status database=\"online\" api=\"online\"/>
            </api>
        </osm>
    "))
}

#[get("/0.6/capabilities")]
fn xml_06capabilities(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    Ok(String::from("
        <osm version=\"0.6\" generator=\"Hecate Server\">
            <api>
                <version minimum=\"0.6\" maximum=\"0.6\"/>
                <area maximum=\"0.25\"/>
                <waynodes maximum=\"2000\"/>
                <changesets maximum_elements=\"10000\"/>
                <timeout seconds=\"300\"/>
                <status database=\"online\" api=\"online\"/>
            </api>
        </osm>
    "))
}

#[get("/0.6/user/details")]
fn xml_user(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    Ok(String::from("
        <osm version=\"0.6\" generator=\"Hecate Server\">
            <user id=\"1\" display_name=\"user\" account_created=\"2010-06-18T12:34:58Z\">
                <description></description>
                <languages><lang>en-US</lang><lang>en</lang></languages>
                <messages>
                    <recieved county=\"0\" unread=\"0\"/>
                    <send count=\"0\"/>
                </messages>
            </user>
        </osm>
    "))
}

#[post("/data/feature", format="application/json", data="<body>")]
fn feature_action(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, schema: State<Option<serde_json::value::Value>>, body: String) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;

    auth_rules.allows_feature_create(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    let mut feat = match body.parse::<GeoJson>() {
        Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Body must be valid GeoJSON Feature")))); },
        Ok(geo) => match geo {
            GeoJson::Feature(feat) => feat,
            _ => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Body must be valid GeoJSON Feature")))); }
        }
    };

    match feature::is_force(&feat) {
        Err(err) => {
            return Err(status::Custom(HTTPStatus::ExpectationFailed, Json(err.as_json())));
        },
        Ok(force) => {
            if force {
                auth_rules.allows_feature_force(&mut auth, &conn)?;
            }
        }
    };

    let delta_message = match feat.foreign_members {
        None => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Feature Must have message property for delta")))); }
        Some(ref members) => match members.get("message") {
            Some(message) => match message.as_str() {
                Some(message) => String::from(message),
                None => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Feature Must have message property for delta")))); }
            },
            None => { return Err(status::Custom(HTTPStatus::BadRequest, Json(json!("Feature Must have message property for delta")))); }
        }
    };

    let trans = conn.transaction().unwrap();

    let mut map: HashMap<String, Option<String>> = HashMap::new();
    map.insert(String::from("message"), Some(delta_message));
    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, Json(json!("Could not create delta"))));
        }
    };

    match feature::action(&trans, schema.inner(), &feat, &None) {
        Ok(res) => {
            if res.new != None {
                feat.id = Some(json!(res.new))
            }
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::ExpectationFailed, Json(err.as_json())));
        }
    }

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat ],
        foreign_members: None,
    };

    if delta::modify(&delta_id, &trans, &fc, &uid).is_err() {
        trans.set_rollback();
        trans.finish().unwrap();
        return Err(status::Custom(HTTPStatus::InternalServerError, Json(json!("Could not create delta"))));
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Json(json!(true)))
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Err(status::Custom(HTTPStatus::InternalServerError, Json(json!(err.to_string()))))
        }
    }
}

#[get("/data/feature/<id>")]
fn feature_get(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<String, status::Custom<Json<serde_json::Value>>> {
    auth_rules.allows_feature_get(&mut auth, &conn.get()?)?;

    match feature::get(&read_conn.get()?, &id) {
        Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(err.as_json())))
    }
}

#[derive(FromForm)]
struct FeatureQuery {
    key: String
}

#[get("/data/feature?<fquery..>")]
fn feature_query(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, fquery: Form<FeatureQuery>) -> Result<String, status::Custom<Json<serde_json::Value>>> {
    auth_rules.allows_feature_get(&mut auth, &conn.get()?)?;

    match feature::query_by_key(&read_conn.get()?, &fquery.key) {
        Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(err.as_json())))
    }
}

#[get("/data/feature/<id>/history")]
fn feature_get_history(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, status::Custom<Json<serde_json::Value>>> {
    let conn = conn.get()?;
    auth_rules.allows_feature_history(&mut auth, &conn)?;

    match delta::history(&conn, &id) {
        Ok(features) => Ok(Json(features)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, Json(json!(err.to_string()))))
    }
}
