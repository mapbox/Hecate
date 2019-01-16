#![feature(proc_macro_hygiene, decl_macro, plugin, custom_derive, custom_attribute)]

static VERSION: &'static str = "0.58.1";

#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate quick_xml;
extern crate postgres;
extern crate postgis;
extern crate protobuf;
extern crate rand;
extern crate valico;
extern crate rocket_contrib;
extern crate geojson;
extern crate env_logger;
extern crate chrono;

pub mod err;
pub mod meta;
pub mod stats;
pub mod delta;
pub mod mvt;
pub mod feature;
pub mod bounds;
pub mod clone;
pub mod stream;
pub mod style;
pub mod osm;
pub mod user;
pub mod auth;

use auth::ValidAuth;
use err::HecateError;

//Postgres Connection Pooling
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use mvt::Encode;

use rand::prelude::*;

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use rocket::http::Status as HTTPStatus;
use rocket::config::{Config, Environment, LoggingLevel, Limits};
use rocket::http::{Cookie, Cookies};
use rocket::{State, Data};
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
                Err(err_msg) => {
                    println!("ERROR: {}", err_msg);
                    std::process::exit(1);
                },
                Ok(_) => ()
            };

            auth
        }
    };

    let db_read: DbRead = DbRead::new(Some(database_read.iter().map(|db| init_pool(&db)).collect()));

    let limits = Limits::new()
        .limit("json", 20971520)
        .limit("forms", 131072);

    let mut config = Config::build(Environment::Production)
        .address("0.0.0.0")
        .log_level(LoggingLevel::Debug)
        .port(port.unwrap_or(8000))
        .limits(limits)
        .workers(workers.unwrap_or(12))
        .unwrap();

    match std::env::var("HECATE_SECRET") {
        Ok(secret) => match config.set_secret_key(secret) {
            Err(_) => {
                println!("ERROR: Invalid Base64 Encoded 256 Bit Secret Key");
                std::process::exit(1);
            },
            _ => println!("Using HECATE_SECRET")
        }
        _ => ()
    };

    rocket::custom(config)
        .manage(DbReadWrite::new(init_pool(&database)))
        .manage(db_read)
        .manage(schema)
        .manage(auth_rules)
        .mount("/", routes![
            index
        ])
        .mount("/admin", routes![
            staticsrv,
            staticsrvredirect
        ])
        .mount("/api", routes![
            server,
            meta_list,
            meta_get,
            meta_delete,
            meta_set,
            schema_get,
            auth_get,
            stats_get,
            stats_regen,
            mvt_get,
            mvt_meta,
            mvt_wipe,
            mvt_regen,
            users,
            user_self,
            user_info,
            user_create,
            user_set_admin,
            user_delete_admin,
            user_create_session,
            user_delete_session,
            style_create,
            style_patch,
            style_public,
            style_private,
            style_delete,
            style_get,
            style_list_public,
            style_list_user,
            delta,
            delta_list,
            feature_action,
            features_action,
            feature_get,
            feature_query,
            feature_get_history,
            features_get,
            bounds,
            bounds_stats,
            bounds_get,
            bounds_set,
            bounds_delete,
            clone_get,
            clone_query,
            osm_capabilities,
            osm_06capabilities,
            osm_user,
            osm_map,
            osm_changeset_create,
            osm_changeset_modify,
            osm_changeset_upload,
            osm_changeset_close
        ])
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
        Err(_) => {
            println!("ERROR: Failed to connect to database");
            std::process::exit(1);
        }
    }
}

pub struct DbRead(pub Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager>>>);
impl DbRead {
    fn new(database: Option<Vec<r2d2::Pool<r2d2_postgres::PostgresConnectionManager>>>) -> Self {
        DbRead(database)
    }

    fn get(&self) -> Result<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, HecateError> {
        match self.0 {
            None => Err(HecateError::new(503, String::from("No Database Read Connection"), None)),
            Some(ref db_read) => {
                let mut rng = thread_rng();
                let db_read_it = rng.gen_range(0, db_read.len());

                match db_read.get(db_read_it).unwrap().get() {
                    Ok(conn) => Ok(conn),
                    Err(_) => Err(HecateError::new(503, String::from("Could not connect to database"), None))
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

    fn get(&self) -> Result<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, HecateError> {
        match self.0.get() {
            Ok(conn) => Ok(conn),
            Err(_) => Err(HecateError::new(503, String::from("Could not connect to database"), None))
        }
    }
}

#[derive(FromForm, Debug)]
struct Filter {
    filter: Option<String>,
    limit: Option<i16>
}

#[catch(401)]
fn not_authorized() -> HecateError {
    HecateError::new(401, String::from("You must be logged in to access this resource"), None)
}

#[catch(404)]
fn not_found() -> HecateError {
    HecateError::new(404, String::from("Resource Not Found"), None)
}


#[get("/")]
fn index() -> &'static str { "Hello World!" }

#[get("/")]
fn server(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    auth_rules.allows_server(&mut auth, &conn.get()?)?;

    Ok(Json(json!({
        "version": VERSION
    })))
}

#[get("/meta")]
fn meta_list(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_meta_list(&mut auth, &conn)?;

    meta::list(&conn)?;

    Ok(Json(json!(meta::list(&conn)?)))
}

#[get("/meta/<key>")]
fn meta_get(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>, key: String) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_meta_get(&mut auth, &conn)?;

    Ok(Json(json!(meta::get(&conn, &key)?)))
}

#[delete("/meta/<key>")]
fn meta_delete(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>, key: String) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_meta_set(&mut auth, &conn)?;

    Ok(Json(json!(meta::delete(&conn, &key)?)))
}

#[post("/meta/<key>", format="application/json", data="<body>")]
fn meta_set(mut auth: auth::Auth, conn: State<DbReadWrite>, auth_rules: State<auth::CustomAuth>, key: String, body: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_meta_set(&mut auth, &conn)?;

    Ok(Json(json!(meta::set(&conn, &key, &body)?)))
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
fn mvt_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Response<'static>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_mvt_get(&mut auth, &conn)?;

    if z > 14 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    let tile = mvt::get(&conn, z, x, y, false)?;

    let mut c = Cursor::new(Vec::new());
    match tile.to_writer(&mut c) {
        Ok(_) => (),
        Err(err) => { return Err(HecateError::new(500, err.to_string(), None)); }
    }

    let mut mvt_response = Response::new();
    mvt_response.set_status(HTTPStatus::Ok);
    mvt_response.set_sized_body(c);
    mvt_response.set_raw_header("Content-Type", "application/x-protobuf");
    Ok(mvt_response)
}

#[get("/tiles/<z>/<x>/<y>/meta")]
fn mvt_meta(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_mvt_meta(&mut auth, &conn)?;

    if z > 14 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    Ok(Json(mvt::meta(&conn, z, x, y)?))
}


#[delete("/tiles")]
fn mvt_wipe(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_mvt_delete(&mut auth, &conn)?;

    Ok(Json(mvt::wipe(&conn)?))
}

#[get("/tiles/<z>/<x>/<y>/regen")]
fn mvt_regen(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Response<'static>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_mvt_regen(&mut auth, &conn)?;

    if z > 14 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    let tile = mvt::get(&conn, z, x, y, true)?;

    let mut c = Cursor::new(Vec::new());
    match tile.to_writer(&mut c) {
        Ok(_) => (),
        Err(err) => { return Err(HecateError::new(500, err.to_string(), None)); }
    }

    let mut mvt_response = Response::new();
    mvt_response.set_status(HTTPStatus::Ok);
    mvt_response.set_sized_body(c);
    mvt_response.set_raw_header("Content-Type", "application/x-protobuf");
    Ok(mvt_response)
}

#[derive(FromForm, Debug)]
struct User {
    username: String,
    password: String,
    email: String
}

#[derive(FromForm, Debug)]
struct Map {
    bbox: String
}

#[get("/user/create?<user..>")]
fn user_create(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, user: Form<User>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_user_create(&mut auth, &conn)?;

    Ok(Json(json!(user::create(&conn, &user.username, &user.password, &user.email)?)))
}

#[get("/users?<filter..>")]
fn users(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, filter: Form<Filter>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_user_list(&mut auth, &conn)?;

    match &filter.filter {
        Some(search) => Ok(Json(json!(user::filter(&conn, &search, &filter.limit)?))),
        None => Ok(Json(json!(user::list(&conn, &filter.limit)?)))
    }
}

#[get("/user/<id>")]
fn user_info(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.is_admin(&mut auth, &conn)?;

    Ok(Json(user::info(&conn, &id)?))
}

#[put("/user/<id>/admin")]
fn user_set_admin(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.is_admin(&mut auth, &conn)?;

    Ok(Json(json!(user::set_admin(&conn, &id)?)))
}

#[delete("/user/<id>/admin")]
fn user_delete_admin(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.is_admin(&mut auth, &conn)?;

    Ok(Json(json!(user::delete_admin(&conn, &id)?)))
}

#[get("/user/info")]
fn user_self(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_user_info(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    Ok(Json(user::info(&conn, &uid)?))
}

#[get("/user/session")]
fn user_create_session(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, mut cookies: Cookies) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_user_create_session(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    let token = user::create_token(&conn, &uid)?;

    cookies.add_private(Cookie::build("session", token)
        .path("/")
        .finish()
    );

    Ok(Json(json!(uid)))
}

#[delete("/user/session")]
fn user_delete_session(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, mut cookies: Cookies) -> Result<Json<serde_json::Value>, HecateError> {
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

#[post("/style", format="application/json", data="<body>")]
fn style_create(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, body: Data) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_create(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = match String::from_utf8(body_vec) {
            Ok(body_str) => body_str,
            Err(_) => { return Err(HecateError::new(400, String::from("Invalid JSON - Non-UTF8"), None)); }
        }
    }

    Ok(Json(json!(style::create(&conn, &uid, &body_str)?)))
}

#[post("/style/<id>/public")]
fn style_public(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_set_public(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    Ok(Json(json!(style::access(&conn, &uid, &id, true)?)))
}

#[post("/style/<id>/private")]
fn style_private(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_set_private(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    Ok(Json(json!(style::access(&conn, &uid, &id, false)?)))
}

#[patch("/style/<id>", format="application/json", data="<body>")]
fn style_patch(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64, body: Data) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_patch(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = match String::from_utf8(body_vec) {
            Ok(body_str) => body_str,
            Err(_) => { return Err(HecateError::new(400, String::from("Invalid JSON - Non-UTF8"), None)); }
        }
    }

    Ok(Json(json!(style::update(&conn, &uid, &id, &body_str)?)))
}

#[delete("/style/<id>")]
fn style_delete(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_delete(&mut auth, &conn)?;
    let uid = auth.uid.unwrap();

    Ok(Json(json!(style::delete(&conn, &uid, &id)?)))
}


#[get("/style/<id>")]
fn style_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_get(&mut auth, &conn)?;

    Ok(Json(json!(style::get(&conn, &auth.uid, &id)?)))
}

#[get("/styles")]
fn style_list_public(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_list(&mut auth, &conn)?;

    Ok(Json(json!(style::list_public(&conn)?)))
}

#[get("/styles/<user>")]
fn style_list_user(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, user: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_list(&mut auth, &conn)?;

    match auth.uid {
        Some(uid) => {
            if uid == user {
                Ok(Json(json!(style::list_user(&conn, &user)?)))
            } else {
                Ok(Json(json!(style::list_user_public(&conn, &user)?)))
            }
        },
        _ => {
            Ok(Json(json!(style::list_user_public(&conn, &user)?)))
        }
    }
}

#[derive(FromForm, Debug)]
struct DeltaList {
    offset: Option<i64>,
    limit: Option<i64>,
    start: Option<String>,
    end: Option<String>
}

#[get("/deltas?<opts..>")]
fn delta_list(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, opts: Form<DeltaList>) ->  Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_delta_list(&mut auth, &conn)?;

    if opts.offset.is_none() && opts.limit.is_none() && opts.start.is_none() && opts.end.is_none() {
        Ok(Json(delta::list_by_offset(&conn, None, None)?))
    } else if opts.offset.is_some() && (opts.start.is_some() || opts.end.is_some()) {
        return Err(HecateError::new(400, String::from("Offset cannot be used with start or end"), None));
    } else if opts.start.is_some() || opts.end.is_some() {
        let start: Option<chrono::NaiveDateTime> = match &opts.start {
            None => None,
            Some(start) => {
                match start.parse() {
                    Err(_) => { return Err(HecateError::new(400, String::from("Invalid Start Timestamp"), None)); },
                    Ok(start) => Some(start)
                }
            }
        };

        let end: Option<chrono::NaiveDateTime> = match &opts.end {
            None => None,
            Some(end) => {
                match end.parse() {
                    Err(_) => { return Err(HecateError::new(400, String::from("Invalid end Timestamp"), None)); },
                    Ok(end) => Some(end)
                }
            }
        };

        Ok(Json(delta::list_by_date(&conn, start, end, opts.limit)?))
    } else if opts.offset.is_some() || opts.limit.is_some() {
        Ok(Json(delta::list_by_offset(&conn, opts.offset, opts.limit)?))
    } else {
        return Err(HecateError::new(400, String::from("Invalid Query Params"), None));
    }
}

#[get("/delta/<id>")]
fn delta(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) ->  Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_delta_get(&mut auth, &conn)?;

    Ok(Json(delta::get_json(&conn, &id)?))
}

#[get("/data/bounds?<filter..>")]
fn bounds(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, filter: Form<Filter>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_list(&mut auth, &conn)?;

    match &filter.filter {
        Some(search) => Ok(Json(json!(bounds::filter(&conn, &search, &filter.limit)?))),
        None => Ok(Json(json!(bounds::list(&conn, &filter.limit)?)))
    }
}

#[get("/data/bounds/<bounds>")]
fn bounds_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String) -> Result<Stream<stream::PGStream>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_list(&mut auth, &conn)?;

    Ok(Stream::from(bounds::get(conn, bounds)?))
}

#[post("/data/bounds/<bounds>", format="application/json", data="<body>")]
fn bounds_set(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String, body: Data) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_create(&mut auth, &conn)?;

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = match String::from_utf8(body_vec) {
            Ok(body_str) => body_str,
            Err(_) => { return Err(HecateError::new(400, String::from("Invalid JSON - Non-UTF8"), None)); }
        }
    }

    let geom: serde_json::Value = match serde_json::from_str(&*body_str) {
        Ok(geom) => geom,
        Err(_) => {
            return Err(HecateError::new(400, String::from("Invalid Feature GeoJSON"), None));
        }
    };

    Ok(Json(json!(bounds::set(&conn, &bounds, &geom)?)))
}

#[delete("/data/bounds/<bounds>")]
fn bounds_delete(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_delete(&mut auth, &conn)?;

    Ok(Json(json!(bounds::delete(&conn, &bounds)?)))
}

#[get("/data/bounds/<bounds>/stats")]
fn bounds_stats(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_stats_bounds(&mut auth, &conn)?;

    Ok(Json(bounds::stats_json(conn, bounds)?))
}

#[derive(FromForm, Debug)]
struct CloneQuery {
    query: String,
    limit: Option<i64>
}

#[get("/data/query?<cquery..>")]
fn clone_query(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, cquery: Form<CloneQuery>) -> Result<Stream<stream::PGStream>, HecateError> {
    auth_rules.allows_clone_query(&mut auth, &conn.get()?)?;

    Ok(Stream::from(clone::query(read_conn.get()?, &cquery.query, &cquery.limit)?))
}

#[get("/data/clone")]
fn clone_get(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Stream<stream::PGStream>, HecateError> {
    auth_rules.allows_clone_get(&mut auth, &conn.get()?)?;

    Ok(Stream::from(clone::get(read_conn.get()?)?))
}

#[get("/data/features?<map..>")]
fn features_get(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, map: Form<Map>) -> Result<Stream<stream::PGStream>, HecateError> {
    auth_rules.allows_feature_get(&mut auth, &conn.get()?)?;

    let bbox: Vec<f64> = map.bbox.split(',').map(|s| s.parse().unwrap()).collect();
    Ok(Stream::from(feature::get_bbox_stream(read_conn.get()?, bbox)?))
}

#[get("/schema")]
fn schema_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, schema: State<Option<serde_json::value::Value>>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_schema_get(&mut auth, &conn)?;

    match schema.inner() {
        Some(ref s) => Ok(Json(json!(s))),
        None => Err(HecateError::new(404, String::from("No schema Validation Enforced"), None))
    }
}

#[get("/auth")]
fn auth_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_auth_get(&mut auth, &conn)?;

    Ok(Json(auth_rules.to_json()))
}

#[get("/data/stats")]
fn stats_get(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_stats_get(&mut auth, &conn)?;

    Ok(Json(stats::get_json(&conn)?))
}

#[get("/data/stats/regen")]
fn stats_regen(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_stats_get(&mut auth, &conn)?;

    Ok(Json(json!(stats::regen(&conn)?)))
}

#[post("/data/features", format="application/json", data="<body>")]
fn features_action(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, schema: State<Option<serde_json::value::Value>>, body: Data) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_feature_create(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = match String::from_utf8(body_vec) {
            Ok(body_str) => body_str,
            Err(_) => { return Err(HecateError::new(400, String::from("Invalid JSON - Non-UTF8"), None)); }
        }
    }

    let mut fc = match body_str.parse::<GeoJson>() {
        Err(_) => { return Err(HecateError::new(400, String::from("Body must be valid GeoJSON Feature"), None)); },
        Ok(geo) => match geo {
            GeoJson::FeatureCollection(fc) => fc,
            _ => { return Err(HecateError::new(400, String::from("Body must be valid GeoJSON FeatureCollection"), None)); }
        }
    };

    let delta_message = match fc.foreign_members {
        None => { return Err(HecateError::new(400, String::from("FeatureCollection Must have message property for delta"), None)); }
        Some(ref members) => match members.get("message") {
            Some(message) => match message.as_str() {
                Some(message) => String::from(message),
                None => { return Err(HecateError::new(400, String::from("FeatureCollection Must have message property for delta"), None)); }
            },
            None => { return Err(HecateError::new(400, String::from("FeatureCollection Must have message property for delta"), None)); }
        }
    };

    let trans = match conn.transaction() {
        Ok(trans) => trans,
        Err(err) => { return Err(HecateError::new(500, String::from("Failed to open transaction"), Some(err.to_string()))); }
    };

    let mut map: HashMap<String, Option<String>> = HashMap::new();
    map.insert(String::from("message"), Some(delta_message));

    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(err);
        }
    };

    for feat in &mut fc.features {
        match feature::is_force(&feat) {
            Err(err) => {
                return Err(err);
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
                return Err(err);
            },
            Ok(res) => {
                if res.new.is_some() {
                    feat.id = Some(geojson::feature::Id::Number(serde_json::Number::from(res.new.unwrap())))
                }
            }
        };
    }

    match delta::modify(&delta_id, &trans, &fc, &uid) {
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(err);
        },
        _ => ()
    };

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            if trans.commit().is_err() {
                return Err(HecateError::new(500, String::from("Failed to commit transaction"), None));
            }

            Ok(Json(json!(true)))
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Err(err)
        }
    }
}

#[get("/0.6/map?<map..>")]
fn osm_map(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, map: Form<Map>) -> Result<String, status::Custom<String>> {
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

    let xml_str = match osm::from_features(&fc) {
        Ok(xml_str) => xml_str,
        Err(err) => { return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string())) }
    };

    Ok(xml_str)
}

#[put("/0.6/changeset/create", data="<body>")]
fn osm_changeset_create(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, body: Data) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = match String::from_utf8(body_vec) {
            Ok(body_str) => body_str,
            Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Invalid JSON - Non-UTF8"))); }
        }
    }

    let uid = auth.uid.unwrap();

    let map = match osm::to_delta(&body_str) {
        Ok(map) => map,
        Err(err) => { return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string())); }
    };

    let trans = match conn.transaction() {
        Ok(trans) => trans,
        Err(_) => { return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to open transaction"))); }
    };

    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, err.as_string()));
        }
    };

    if trans.commit().is_err() {
        return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to commit transaction")));
    }

    Ok(delta_id.to_string())
}

#[put("/0.6/changeset/<id>/close")]
fn osm_changeset_close(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, id: i64) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    Ok(id.to_string())
}

#[put("/0.6/changeset/<delta_id>", data="<body>")]
fn osm_changeset_modify(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, delta_id: i64, body: Data) -> Result<Response<'static>, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = String::from_utf8(body_vec).unwrap();
    }

    let uid = auth.uid.unwrap();

    let trans = match conn.transaction() {
        Ok(trans) => trans,
        Err(_) => { return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to open transaction"))); }
    };

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

    let map = match osm::to_delta(&body_str) {
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
            return Err(status::Custom(HTTPStatus::InternalServerError, err.as_string()));
        }
    };

    if trans.commit().is_err() {
        return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to commit transaction")));
    }

    Err(status::Custom(HTTPStatus::Ok, delta_id.to_string()))
}

#[post("/0.6/changeset/<delta_id>/upload", data="<body>")]
fn osm_changeset_upload(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, schema: State<Option<serde_json::value::Value>>, delta_id: i64, body: Data) -> Result<Response<'static>, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = String::from_utf8(body_vec).unwrap();
    }

    let uid = auth.uid.unwrap();

    let trans = match conn.transaction() {
        Ok(trans) => trans,
        Err(_) => { return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to open transaction"))); }
    };

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

    let (mut fc, tree) = match osm::to_features(&body_str) {
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
                    feat.id = Some(geojson::feature::Id::Number(serde_json::Number::from(feat_res.new.unwrap())));
                }

                feat_res
            }
        };

        ids.insert(feat_res.old.unwrap(), feat_res);
    }

    let diffres = match osm::to_diffresult(ids, tree) {
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
            if trans.commit().is_err() {
                return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to commit transaction")));
            }

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
fn osm_capabilities(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
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
fn osm_06capabilities(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
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
fn osm_user(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
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
fn feature_action(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: State<DbReadWrite>, schema: State<Option<serde_json::value::Value>>, body: Data) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_feature_create(&mut auth, &conn)?;

    let uid = auth.uid.unwrap();

    let body_str: String;
    {
        let mut body_stream = body.open();
        let mut body_vec = Vec::new();

        let mut buffer = [0; 1024];
        let mut buffer_size: usize = 1;

        while buffer_size > 0 {
            buffer_size = body_stream.read(&mut buffer[..]).unwrap_or(0);
            body_vec.append(&mut buffer[..buffer_size].to_vec());
        }

        body_str = String::from_utf8(body_vec).unwrap();
    }

    let mut feat = match body_str.parse::<GeoJson>() {
        Err(_) => { return Err(HecateError::new(400, String::from("Body must be valid GeoJSON Feature"), None)); }
        Ok(geo) => match geo {
            GeoJson::Feature(feat) => feat,
            _ => { return Err(HecateError::new(400, String::from("Body must be valid GeoJSON Feature"), None)); }
        }
    };

    if feature::is_force(&feat)? {
        auth_rules.allows_feature_force(&mut auth, &conn)?;
    };

    let delta_message = match feat.foreign_members {
        None => { return Err(HecateError::new(400, String::from("Feature Must have message property for delta"), None)); }
        Some(ref members) => match members.get("message") {
            Some(message) => match message.as_str() {
                Some(message) => String::from(message),
                None => { return Err(HecateError::new(400, String::from("Feature Must have message property for delta"), None)); }
            },
            None => { return Err(HecateError::new(400, String::from("Feature Must have message property for delta"), None)); }
        }
    };

    let trans = match conn.transaction() {
        Ok(trans) => trans,
        Err(err) => { return Err(HecateError::new(500, String::from("Failed to open transaction"), Some(err.to_string()))); }
    };

    let mut map: HashMap<String, Option<String>> = HashMap::new();
    map.insert(String::from("message"), Some(delta_message));
    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(err);
        }
    };

    match feature::action(&trans, schema.inner(), &feat, &None) {
        Ok(res) => {
            if res.new.is_some() {
                feat.id = Some(geojson::feature::Id::Number(serde_json::Number::from(res.new.unwrap())));
            }
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(err);
        }
    }

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat ],
        foreign_members: None,
    };

    match delta::modify(&delta_id, &trans, &fc, &uid) {
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(err);
        },
        _ => ()
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            if trans.commit().is_err() {
                return Err(HecateError::new(500, String::from("Failed to commit transaction"), None));
            }

            Ok(Json(json!(true)))
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Err(err)
        }
    }
}

#[get("/data/feature/<id>")]
fn feature_get(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<String, HecateError> {
    auth_rules.allows_feature_get(&mut auth, &conn.get()?)?;

    match feature::get(&read_conn.get()?, &id) {
        Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
        Err(err) => Err(err)
    }
}

#[derive(FromForm, Debug)]
struct FeatureQuery {
    key: Option<String>,
    point: Option<String>
}

#[get("/data/feature?<fquery..>")]
fn feature_query(conn: State<DbReadWrite>, read_conn: State<DbRead>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, fquery: Form<FeatureQuery>) -> Result<Json<serde_json::Value>, HecateError> {
    auth_rules.allows_feature_get(&mut auth, &conn.get()?)?;

    if fquery.key.is_some() && fquery.point.is_some() {
        Err(HecateError::new(400, String::from("key and point params cannot be used together"), None))
    } else if fquery.key.is_some() {
        Ok(Json(feature::query_by_key(&read_conn.get()?, &fquery.key.as_ref().unwrap())?))
    } else if fquery.point.is_some() {
        Ok(Json(feature::query_by_point(&read_conn.get()?, &fquery.point.as_ref().unwrap())?))
    } else {
        Err(HecateError::new(400, String::from("key or point param must be used"), None))
    }
}

#[get("/data/feature/<id>/history")]
fn feature_get_history(conn: State<DbReadWrite>, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_feature_history(&mut auth, &conn)?;

    Ok(Json(delta::history(&conn, &id)?))
}
