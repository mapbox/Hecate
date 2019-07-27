pub static VERSION: &'static str = "0.71.1";
pub static POSTGRES: f64 = 10.0;
pub static POSTGIS: f64 = 2.4;

#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

pub mod err;
pub mod validate;
pub mod meta;
pub mod stats;
pub mod db;
pub mod stream;
pub mod bounds;
pub mod delta;
pub mod mvt;
pub mod feature;
pub mod clone;
pub mod style;
pub mod worker;
pub mod webhooks;
pub mod osm;
pub mod user;
pub mod auth;


use actix_web::{web, web::Json, App, HttpResponse, HttpServer, Responder, middleware};
use rand::prelude::*;
use geojson::GeoJson;
use crate::{
    auth::ValidAuth,
    err::HecateError,
    db::*
};
use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
    collections::HashMap
};

pub fn start(
    database: Database,
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

    let auth_rules = auth::AuthContainer(auth_rules);

    let db_replica = DbReplica::new(Some(database.replica.iter().map(|db| db::init_pool(&db)).collect()));
    let db_sandbox = DbSandbox::new(Some(database.sandbox.iter().map(|db| db::init_pool(&db)).collect()));
    let db_main = DbReadWrite::new(init_pool(&database.main));

    let worker = worker::Worker::new(database.main.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(auth_rules.clone())
            .data(worker.clone())
            .data(db_replica.clone())
            .data(db_sandbox.clone())
            .data(db_main.clone())
            .data(schema.clone())
            //TODO HANDLE GENERIC 404/401
            .route("/", web::get().to(index))
            .service(actix_files::Files::new("/admin", "./web/dist/"))
            .service(web::scope("api")
                .service(web::resource("/")
                    .route(web::get().to(server))
                )
                .service(web::resource("meta")
                     .route(web::get().to(meta_list))
                )
                .service(web::resource("meta/{key}")
                    .route(web::post().to(meta_set))
                    .route(web::delete().to(meta_delete))
                    .route(web::get().to(meta_get))
                )
                .service(web::resource("schema")
                    .route(web::get().to(schema_get))
                )
                .service(web::scope("webhooks")
                    .service(web::resource("")
                        .route(web::get().to(webhooks_list))
                        .route(web::post().to(webhooks_create))
                    )
                    .service(web::resource("{id}")
                        .route(web::get().to(webhooks_get))
                        .route(web::delete().to(webhooks_delete))
                        .route(web::post().to(webhooks_update))
                    )
                )
                .service(web::scope("tiles")
                    .service(web::resource("")
                        .route(web::delete().to(mvt_wipe))
                    )
                    .service(web::resource("{z}/{x}/{y}")
                        .route(web::get().to(mvt_get))
                    )
                    .service(web::resource("{z}/{x}/{y}/meta")
                        .route(web::get().to(mvt_meta))
                    )
                    .service(web::resource("{z}/{x}/{y}/regen")
                        .route(web::get().to(mvt_regen))
                    )
                )
                .service(web::resource("users")
                    .route(web::get().to(users))
                )
                .service(web::scope("user")
                    .service(web::resource("create")
                        .route(web::get().to(user_create))
                    )
                    .service(web::resource("{uid}")
                        .route(web::get().to(user_info))
                    )
                    .service(web::resource("{uid}/admin")
                        .route(web::put().to(user_set_admin))
                        .route(web::delete().to(user_delete_admin))
                    )
                )
                .service(web::scope("data")
                    .service(web::resource("stats")
                        .route(web::get().to(stats_get))
                    )
                    .service(web::resource("stats/regen")
                        .route(web::get().to(stats_regen))
                    )
                    /*
                    .service(web::resource("clone")
                        .route(web::get().to(clone_get))
                    )
                    */
                )
            )
    })
        .workers(workers.unwrap_or(12) as usize)
        .bind(format!("0.0.0.0:{}", port.unwrap_or(8000)).as_str())
        .unwrap()
        .run()
        .unwrap();

    /*
    .mount("/admin", routes![
        staticsrvredirect
    ])
    .mount("/api", routes![
        auth_get,
        user_self,
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
        features_query,
        bounds,
        bounds_stats,
        bounds_meta,
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
            */
}

#[derive(Deserialize, Debug)]
struct Filter {
    filter: Option<String>,
    limit: Option<i16>
}

#[derive(Deserialize, Debug)]
struct Map {
    bbox: Option<String>,
    point: Option<String>
}

#[derive(Deserialize, Debug)]
struct DeltaList {
    offset: Option<i64>,
    limit: Option<i64>,
    start: Option<String>,
    end: Option<String>
}

#[derive(Deserialize, Debug)]
struct CloneQuery {
    query: String,
    limit: Option<i64>
}

#[derive(Deserialize, Debug)]
struct FeatureQuery {
    key: Option<String>,
    point: Option<String>
}

fn not_authorized() -> HecateError {
    HecateError::new(401, String::from("You must be logged in to access this resource"), None)
}

fn not_found() -> HecateError {
    HecateError::new(404, String::from("Resource Not Found"), None)
}

fn index() -> &'static str { "Hello World!" }

fn server(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    //auth_rules.allows_server(&mut auth, &*conn.get()?)?;

    Ok(Json(json!({
        "version": VERSION
    })))
}

fn meta_list(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> actix_web::Result<impl Responder> {
    let conn = conn.get()?;

    //auth_rules.allows_meta_list(&mut auth, &*conn)?;

    let list = serde_json::to_value(meta::list(&*conn)?).unwrap();

    Ok(Json(list))
}


fn meta_get(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    key: String
) -> actix_web::Result<Json<serde_json::Value>> {
    let conn = conn.get()?;

    //auth_rules.allows_meta_get(&mut auth, &*conn)?;
    worker.queue(worker::Task::new(worker::TaskType::Meta));

    Ok(Json(json!(meta::Meta::get(&*conn, &key)?)))
}


fn meta_delete(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    key: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_meta_set(&mut auth, &*conn)?;

    worker.queue(worker::Task::new(worker::TaskType::Meta));

    Ok(Json(json!(meta::delete(&*conn, &key)?)))
}

fn meta_set(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    key: String,
    meta: Json<meta::Meta>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    //auth_rules.allows_meta_set(&mut auth, &*conn)?;

    worker.queue(worker::Task::new(worker::TaskType::Meta));

    Ok(Json(json!(meta.set(&*conn)?)))
}

/*

#[get("/")]
fn staticsrvredirect() -> rocket::response::Redirect {
    rocket::response::Redirect::to("/admin/index.html")
}

*/

fn mvt_get(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    z: String, x: String, y: String
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;

    let z: u8 = match z.parse() {
        Ok(z) => z,
        Err(err) => { return Err(HecateError::new(400, String::from("z coordinate must be integer"), None)); }
    };
    let x: u32 = match x.parse() {
        Ok(x) => x,
        Err(err) => { return Err(HecateError::new(400, String::from("x coordinate must be integer"), None)); }
    };
    let y: u32 = match y.parse() {
        Ok(y) => y,
        Err(err) => { return Err(HecateError::new(400, String::from("y coordinate must be integer"), None)); }
    };

    //auth_rules.allows_mvt_get(&mut auth, &*conn)?;

    if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    let tile = mvt::get(&*conn, z, x, y, false)?;

    Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
        .content_type("application/x-protobuf")
        .content_length(tile.len() as u64)
        .body(tile))
}


fn mvt_meta(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    z: String, x: String, y: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    //auth_rules.allows_mvt_meta(&mut auth, &*conn)?;

    let z: u8 = match z.parse() {
        Ok(z) => z,
        Err(err) => { return Err(HecateError::new(400, String::from("z coordinate must be integer"), Some(err.to_string()))); }
    };
    let x: u32 = match x.parse() {
        Ok(x) => x,
        Err(err) => { return Err(HecateError::new(400, String::from("x coordinate must be integer"), Some(err.to_string()))); }
    };
    let y: u32 = match y.parse() {
        Ok(y) => y,
        Err(err) => { return Err(HecateError::new(400, String::from("y coordinate must be integer"), Some(err.to_string()))); }
    };

    if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    Ok(Json(mvt::meta(&*conn, z, x, y)?))
}

fn mvt_wipe(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    //auth_rules.allows_mvt_delete(&mut auth, &*conn)?;

    Ok(Json(mvt::wipe(&*conn)?))
}

fn mvt_regen(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    z: String, x: String, y: String
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;
    //auth_rules.allows_mvt_regen(&mut auth, &*conn)?;

    let z: u8 = match z.parse() {
        Ok(z) => z,
        Err(err) => { return Err(HecateError::new(400, String::from("z coordinate must be integer"), None)); }
    };
    let x: u32 = match x.parse() {
        Ok(x) => x,
        Err(err) => { return Err(HecateError::new(400, String::from("x coordinate must be integer"), None)); }
    };
    let y: u32 = match y.parse() {
        Ok(y) => y,
        Err(err) => { return Err(HecateError::new(400, String::from("y coordinate must be integer"), None)); }
    };

    if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    let tile = mvt::get(&*conn, z, x, y, true)?;

    Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
        .content_type("application/x-protobuf")
        .content_length(tile.len() as u64)
        .body(tile))
}

fn user_create(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    user: web::Form<user::User>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    //auth_rules.allows_user_create(&mut auth, &*conn)?;

    user.set(&*conn)?;

    worker.queue(worker::Task::new(worker::TaskType::User(user.username.clone())));

    Ok(Json(json!(true)))
}

fn users(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    filter: Json<Option<Filter>>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_user_list(&mut auth, &*conn)?;

    match filter.into_inner() {
        None => Ok(Json(user::user::list(&*conn, &None)?)),
        Some(filter) => match &filter.filter {
            Some(search) => Ok(Json(json!(user::user::filter(&*conn, &search, &filter.limit)?))),
            None => Ok(Json(user::user::list(&*conn, &None)?))
        }
    }
}

fn user_info(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.is_admin(&mut auth, &*conn)?;

    let uid: i64 = match uid.parse() {
        Ok(uid) => uid,
        Err(err) => {
            return Err(HecateError::new(400, String::from("User ID must be an integer"), Some(err.to_string())));
        }
    };

    let user = user::User::get(&*conn, &uid)?.to_value();

    Ok(Json(user))
}

fn user_set_admin(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.is_admin(&mut auth, &*conn)?;

    let uid: i64 = match uid.parse() {
        Ok(uid) => uid,
        Err(err) => {
            return Err(HecateError::new(400, String::from("User ID must be an integer"), Some(err.to_string())));
        }
    };

    let mut user = user::User::get(&*conn, &uid)?;

    if user.is_admin() {
        return Err(HecateError::new(400, format!("{} is already an admin", user.username), None));
    }

    user.admin(true);
    user.set(&*conn)?;

    Ok(Json(json!(true)))
}

fn user_delete_admin(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.is_admin(&mut auth, &*conn)?;

    let uid: i64 = match uid.parse() {
        Ok(uid) => uid,
        Err(err) => {
            return Err(HecateError::new(400, String::from("User ID must be an integer"), Some(err.to_string())));
        }
    };

    let mut user = user::User::get(&*conn, &uid)?;

    if !user.is_admin() {
        return Err(HecateError::new(400, format!("{} is not an admin", user.username), None));
    }

    user.admin(false);
    user.set(&*conn)?;

    Ok(Json(json!(true)))
}

/*

#[get("/user/info")]
fn user_self(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_user_info(&mut auth, &*conn)?;

    let uid = auth.uid.unwrap();

    Ok(Json(user::info(&*conn, &uid)?))
}

#[get("/user/session")]
fn user_create_session(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    mut cookies: Cookies
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_user_create_session(&mut auth, &*conn)?;

    let uid = auth.uid.unwrap();

    let token = user::create_token(&*conn, &uid)?;

    cookies.add(Cookie::build("session", token)
        .path("/")
        .http_only(true)
        .finish()
    );

    Ok(Json(json!(uid)))
}

#[delete("/user/session")]
fn user_delete_session(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    mut cookies: Cookies
) -> Result<Json<serde_json::Value>, HecateError> {
    // there is no auth check here for deleting tokens, the web interface should
    // always be able to de-authenticate to prevent errors

    let token = match cookies.get("session") {
        Some(session) => Some(String::from(session.value())),
        None => None
    };

    cookies.remove(Cookie::build("session", String::from(""))
        .path("/")
        .http_only(true)
        .finish()
    );

    match token {
        Some(token) => {
            let uid = match auth.uid {
                Some(uid) => uid,
                None => { return Ok(Json(json!(true))); }
            };

            match user::destroy_token(&*conn.get()?, &uid, &token) {
                _ => {
                    Ok(Json(json!(true)))
                }
            }
        },
        None => Ok(Json(json!(true)))
    }
}

#[post("/style", format="application/json", data="<body>")]
fn style_create(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    body: Data
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_create(&mut auth, &*conn)?;
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

    let style_id = style::create(&*conn, &uid, &body_str)?;
    worker.queue(worker::Task::new(worker::TaskType::Style(style_id)));

    Ok(Json(json!(style_id)))
}

#[post("/style/<id>/public")]
fn style_public(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: i64
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_set_public(&mut auth, &*conn)?;
    let uid = auth.uid.unwrap();

    Ok(Json(json!(style::access(&*conn, &uid, &id, true)?)))
}

#[post("/style/<id>/private")]
fn style_private(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: i64
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_set_private(&mut auth, &*conn)?;
    let uid = auth.uid.unwrap();

    Ok(Json(json!(style::access(&*conn, &uid, &id, false)?)))
}

#[patch("/style/<id>", format="application/json", data="<body>")]
fn style_patch(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    id: i64,
    body: Data
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_patch(&mut auth, &*conn)?;
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

    worker.queue(worker::Task::new(worker::TaskType::Style(id)));

    Ok(Json(json!(style::update(&*conn, &uid, &id, &body_str)?)))
}

#[delete("/style/<id>")]
fn style_delete(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    id: i64
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_delete(&mut auth, &*conn)?;
    let uid = auth.uid.unwrap();

    worker.queue(worker::Task::new(worker::TaskType::Style(id)));

    Ok(Json(json!(style::delete(&*conn, &uid, &id)?)))
}


#[get("/style/<id>")]
fn style_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: i64
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_get(&mut auth, &*conn)?;

    Ok(Json(json!(style::get(&*conn, &auth.uid, &id)?)))
}

#[get("/styles")]
fn style_list_public(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_list(&mut auth, &*conn)?;

    Ok(Json(json!(style::list_public(&*conn)?)))
}

#[get("/styles/<user>")]
fn style_list_user(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    user: i64
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_style_list(&mut auth, &*conn)?;

    match auth.uid {
        Some(uid) => {
            if uid == user {
                Ok(Json(json!(style::list_user(&*conn, &user)?)))
            } else {
                Ok(Json(json!(style::list_user_public(&*conn, &user)?)))
            }
        },
        _ => {
            Ok(Json(json!(style::list_user_public(&*conn, &user)?)))
        }
    }
}

#[get("/deltas?<opts..>")]
fn delta_list(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    opts: Json<DeltaList>
) ->  Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_delta_list(&mut auth, &*conn)?;

    if opts.offset.is_none() && opts.limit.is_none() && opts.start.is_none() && opts.end.is_none() {
        Ok(Json(delta::list_by_offset(&*conn, None, None)?))
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

        Ok(Json(delta::list_by_date(&*conn, start, end, opts.limit)?))
    } else if opts.offset.is_some() || opts.limit.is_some() {
        Ok(Json(delta::list_by_offset(&*conn, opts.offset, opts.limit)?))
    } else {
        return Err(HecateError::new(400, String::from("Invalid Query Params"), None));
    }
}

#[get("/delta/<id>")]
fn delta(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: i64
) ->  Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_delta_get(&mut auth, &*conn)?;

    Ok(Json(delta::get_json(&*conn, &id)?))
}

#[get("/data/bounds?<filter..>")]
fn bounds(
    conn: web::Data<DbReplica>,
    mut auth:
    auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    filter: Json<Filter>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_list(&mut auth, &*conn)?;

    match &filter.filter {
        Some(search) => Ok(Json(json!(bounds::filter(&*conn, &search, &filter.limit)?))),
        None => Ok(Json(json!(bounds::list(&*conn, &filter.limit)?)))
    }
}

#[get("/data/bounds/<bounds>")]
fn bounds_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: String
) -> Result<Stream<stream::PGStream>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_list(&mut auth, &*conn)?;

    Ok(Stream::from(bounds::get(conn, bounds)?))
}

#[post("/data/bounds/<bounds>", format="application/json", data="<body>")]
fn bounds_set(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: String,
    body: Data
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_create(&mut auth, &*conn)?;

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

    Ok(Json(json!(bounds::set(&*conn, &bounds, &geom)?)))
}

#[delete("/data/bounds/<bounds>")]
fn bounds_delete(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_delete(&mut auth, &*conn)?;

    Ok(Json(json!(bounds::delete(&*conn, &bounds)?)))
}
*/

fn webhooks_list(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_webhooks_list(&mut auth, &*conn)?;

    match serde_json::to_value(webhooks::list(&*conn, webhooks::Action::All)?) {
        Ok(hooks) => Ok(Json(hooks)),
        Err(_) => Err(HecateError::new(500, String::from("Internal Server Error"), None))
    }
}

fn webhooks_get(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_webhooks_list(&mut auth, &*conn)?;

    let id: i64 = match id.parse() {
        Ok(id) => id,
        Err(err) => {
            return Err(HecateError::new(400, String::from("Webhook ID must be an integer"), Some(err.to_string())));
        }
    };

    match serde_json::to_value(webhooks::get(&*conn, id)?) {
        Ok(hooks) => Ok(Json(hooks)),
        Err(_) => Err(HecateError::new(500, String::from("Internal Server Error"), None))
    }
}

fn webhooks_delete(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: String
) -> Result<Json<bool>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_webhooks_delete(&mut auth, &*conn)?;

    let id: i64 = match id.parse() {
        Ok(id) => id,
        Err(err) => {
            return Err(HecateError::new(400, String::from("Webhook ID must be an integer"), Some(err.to_string())));
        }
    };

    Ok(Json(webhooks::delete(&*conn, id)?))
}

fn webhooks_create(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    webhook: Json<webhooks::WebHook>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_webhooks_update(&mut auth, &*conn)?;

    match serde_json::to_value(webhooks::create(&*conn, webhook.into_inner())?) {
        Ok(webhook) => Ok(Json(webhook)),
        Err(_) => { return Err(HecateError::new(500, String::from("Failed to return webhook ID"), None)); }
    }
}

fn webhooks_update(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    mut webhook: Json<webhooks::WebHook>,
    id: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_webhooks_update(&mut auth, &*conn)?;

    let id: i64 = match id.parse() {
        Ok(id) => id,
        Err(err) => {
            return Err(HecateError::new(400, String::from("Webhook ID must be an integer"), Some(err.to_string())));
        }
    };

    webhook.id = Some(id);

    match serde_json::to_value(webhooks::update(&*conn, webhook.into_inner())?) {
        Ok(webhook) => Ok(Json(webhook)),
        Err(_) => { return Err(HecateError::new(500, String::from("Failed to return webhook ID"), None)); }
    }
}

/*
#[get("/data/bounds/<bounds>/stats")]
fn bounds_stats(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_stats_bounds(&mut auth, &*conn)?;

    Ok(Json(bounds::stats_json(&*conn, bounds)?))
}

#[get("/data/bounds/<bounds>/meta")]
fn bounds_meta(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: String
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_bounds_get(&mut auth, &*conn)?;

    Ok(Json(bounds::meta(&*conn, bounds)?))
}


#[get("/data/query?<cquery..>")]
fn clone_query(
    sandbox_conn: web::Data<DbSandbox>,
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    cquery: Json<CloneQuery>
) -> Result<Stream<stream::PGStream>, HecateError> {
    auth_rules.allows_clone_query(&mut auth, &*conn.get()?)?;

    Ok(Stream::from(clone::query(sandbox_conn.get()?, &cquery.query, &cquery.limit)?))
}

fn clone_get(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<stream::PGStream, HecateError> {
    //auth_rules.allows_clone_get(&mut auth, &*conn.get()?)?;

    Ok(clone::get(conn.get()?)?)
}

#[get("/data/features?<map..>")]
fn features_query(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    map: Json<Map>
) -> Result<Stream<stream::PGStream>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_feature_get(&mut auth, &*conn)?;

    if map.bbox.is_some() && map.point.is_some() {
        Err(HecateError::new(400, String::from("key and point params cannot be used together"), None))
    } else if map.bbox.is_some() {
        let bbox: Vec<f64> = map.bbox.as_ref().unwrap().split(',').map(|s| s.parse().unwrap()).collect();
        Ok(Stream::from(feature::get_bbox_stream(conn, &bbox)?))
    } else if map.point.is_some() {
        Ok(Stream::from(feature::get_point_stream(conn, &map.point.as_ref().unwrap())?))
    } else {
        Err(HecateError::new(400, String::from("key or point param must be used"), None))
    }

}
*/

fn schema_get(
    conn: web::Data<DbReplica>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    schema: web::Data<Option<serde_json::value::Value>>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_schema_get(&mut auth, &*conn)?;

    match schema.get_ref() {
        Some(s) => Ok(Json(json!(s))),
        None => Err(HecateError::new(404, String::from("No schema Validation Enforced"), None))
    }
}

/*

#[get("/auth")]
fn auth_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_auth_get(&mut auth, &*conn)?;

    Ok(Json(auth_rules.to_json()))
}

*/

fn stats_get(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_stats_get(&mut auth, &*conn)?;

    Ok(Json(stats::get_json(&*conn)?))
}

fn stats_regen(
    conn: web::Data<DbReadWrite>,
    //mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    //auth_rules.allows_stats_get(&mut auth, &*conn)?;

    Ok(Json(json!(stats::regen(&*conn)?)))
}

/*
#[post("/data/features", format="application/json", data="<body>")]
fn features_action(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    worker: web::Data<worker::Worker>,
    schema: web::Data<Option<serde_json::value::Value>>,
    body: Data
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_feature_create(&mut auth, &*conn)?;

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
                    auth_rules.allows_feature_force(&mut auth, &*conn)?;
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

            worker.queue(worker::Task::new(worker::TaskType::Delta(delta_id)));

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
fn osm_map(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    map: Json<Map>
) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    let query: Vec<f64> = map.bbox.as_ref().unwrap().split(',').map(|s| s.parse().unwrap()).collect();

    let fc = match feature::get_bbox(&*conn, query) {
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
fn osm_changeset_create(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    body: Data
) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
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
            return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
        }
    };

    if trans.commit().is_err() {
        return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to commit transaction")));
    }

    Ok(delta_id.to_string())
}

#[put("/0.6/changeset/<id>/close")]
fn osm_changeset_close(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    id: i64
) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
        Ok(_) => (),
        Err(_) => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized"))); }
    };

    Ok(id.to_string())
}

#[put("/0.6/changeset/<delta_id>", data="<body>")]
fn osm_changeset_modify(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    delta_id: i64,
    body: Data
) -> Result<Response<'static>, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
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
            return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
        }
    };

    if trans.commit().is_err() {
        return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to commit transaction")));
    }

    Err(status::Custom(HTTPStatus::Ok, delta_id.to_string()))
}

#[post("/0.6/changeset/<delta_id>/upload", data="<body>")]
fn osm_changeset_upload(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    schema: web::Data<Option<serde_json::value::Value>>,
    worker: web::Data<worker::Worker>,
    delta_id: i64,
    body: Data
) -> Result<Response<'static>, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
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

            worker.queue(worker::Task::new(worker::TaskType::Delta(delta_id)));

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
fn osm_capabilities(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
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
fn osm_06capabilities(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
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
fn osm_user(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<String, status::Custom<String>> {
    let conn = conn.get().unwrap();

    match auth_rules.allows_osm_get(&mut auth, &*conn) {
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
fn feature_action(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    schema: web::Data<Option<serde_json::value::Value>>,
    worker: web::Data<worker::Worker>,
    body: Data
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.allows_feature_create(&mut auth, &*conn)?;

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
        auth_rules.allows_feature_force(&mut auth, &*conn)?;
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

            worker.queue(worker::Task::new(worker::TaskType::Delta(delta_id)));

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
fn feature_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: i64
) -> Result<Response<'static>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_feature_get(&mut auth, &*conn)?;

    match feature::get(&*conn, &id) {
        Ok(feature) => {
            let feature = geojson::GeoJson::from(feature).to_string();

            let mut response = Response::new();

            response.set_status(HTTPStatus::Ok);
            response.set_sized_body(Cursor::new(feature));
            response.set_raw_header("Content-Type", "application/json");

            Ok(response)
        },
        Err(err) => Err(err)
    }
}


#[get("/data/feature?<fquery..>")]
fn feature_query(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    fquery: Json<FeatureQuery>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_feature_get(&mut auth, &*conn)?;

    if fquery.key.is_some() && fquery.point.is_some() {
        Err(HecateError::new(400, String::from("key and point params cannot be used together"), None))
    } else if fquery.key.is_some() {
        Ok(Json(feature::query_by_key(&*conn, &fquery.key.as_ref().unwrap())?))
    } else if fquery.point.is_some() {
        let mut results = feature::query_by_point(&*conn, &fquery.point.as_ref().unwrap())?;
        Ok(Json(results.pop().unwrap()))
    } else {
        Err(HecateError::new(400, String::from("key or point param must be used"), None))
    }
}

#[get("/data/feature/<id>/history")]
fn feature_get_history(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: i64
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.allows_feature_history(&mut auth, &*conn)?;

    Ok(Json(delta::history(&*conn, &id)?))
}
*/
