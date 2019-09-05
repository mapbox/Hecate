pub static VERSION: &'static str = "0.72.0";
pub static POSTGRES: f64 = 10.0;
pub static POSTGIS: f64 = 2.4;

///
/// The Maximum number of bytes allowed in
/// a request body
///
pub static MAX_BODY: u64 = 20971520;

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

use actix_http::httpmessage::HttpMessage;
use actix_web::{web, web::Json, App, HttpResponse, HttpRequest, HttpServer, Responder, middleware};
use futures::{Future, Stream, future::Either};
use geojson::GeoJson;
use crate::{
    auth::ValidAuth,
    err::HecateError,
    db::*
};
use std::{
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
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(auth_rules.clone())
            .data(worker.clone())
            .data(db_replica.clone())
            .data(db_sandbox.clone())
            .data(db_main.clone())
            .data(schema.clone())
            //TODO HANDLE GENERIC 404
            .route("/", web::get().to(index))
            .service(
                actix_files::Files::new("/admin", "./web/dist/")
                    .index_file("index.html")
            )
            .service(web::scope("api")
                .service(web::resource("")
                    .route(web::get().to(server))
                )
                .service(web::resource("capabilities")
                    .route(web::get().to(osm_capabilities))
                )
                .service(web::scope("0.6")
                    .service(web::resource("capabilities")
                        .route(web::get().to(osm_capabilities))
                    )
                    .service(web::resource("user/details")
                        .route(web::get().to(osm_user))
                    )
                    .service(web::resource("map")
                        .route(web::get().to(osm_map))
                    )
                    .service(web::resource("changeset/create")
                        .route(web::put().to_async(osm_changeset_create))
                    )
                    .service(web::resource("changeset/{delta_id}")
                        .route(web::put().to_async(osm_changeset_modify))
                    )
                    .service(web::resource("changeset/{delta_id}/close")
                        .route(web::put().to(osm_changeset_close))
                    )
                    .service(web::resource("changeset/{delta_id}/upload")
                        .route(web::post().to_async(osm_changeset_upload))
                    )
                )
                .service(web::resource("auth")
                    .route(web::get().to(auth_get))
                )
                .service(web::resource("meta")
                    .route(web::get().to(meta_list))
                )
                .service(web::resource("meta/{key}")
                    .route(web::post().to(meta_set))
                    .route(web::delete().to(meta_delete))
                    .route(web::get().to(meta_get))
                )
                .service(web::scope("style")
                    .service(web::resource("")
                        .route(web::post().to_async(style_create))
                    )
                    .service(web::resource("{style_id}")
                        .route(web::delete().to(style_delete))
                        .route(web::get().to(style_get))
                        .route(web::patch().to_async(style_patch))
                    )
                    .service(web::resource("{style_id}/public")
                        .route(web::post().to(style_public))
                    )
                    .service(web::resource("{style_id}/private")
                        .route(web::post().to(style_private))
                    )
                 )
                .service(web::scope("styles")
                    .service(web::resource("")
                        .route(web::get().to(style_list_public))
                    )
                    .service(web::resource("{user_id}")
                        .route(web::get().to(style_list_user))
                    )
                 )
                .service(web::resource("schema")
                    .route(web::get().to(schema_get))
                )
                .service(web::resource("deltas")
                    .route(web::get().to(delta_list))
                )
                .service(web::resource("delta/{id}")
                    .route(web::get().to(delta))
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
                    .service(web::resource("info")
                        .route(web::get().to(user_self))
                    )
                    .service(web::resource("session")
                        .route(web::get().to(user_create_session))
                        .route(web::delete().to(user_delete_session))
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
                    .service(web::resource("feature")
                        .route(web::get().to(feature_query))
                        .route(web::post().to_async(feature_action))
                    )
                    .service(web::resource("feature/{id}")
                        .route(web::get().to_async(feature_get))
                    )
                    .service(web::resource("feature/{id}/history")
                        .route(web::get().to(feature_get_history))
                    )
                    .service(web::resource("features")
                        .route(web::post().to_async(features_action))
                        .route(web::get().to(features_query))
                    )
                    .service(web::resource("stats")
                        .route(web::get().to(stats_get))
                    )
                    .service(web::resource("stats/regen")
                        .route(web::get().to(stats_regen))
                    )
                    .service(web::resource("query")
                        .route(web::get().to(clone_query))
                    )
                    .service(web::resource("clone")
                        .route(web::get().to(clone_get))
                    )
                    .service(web::scope("bounds")
                        .service(web::resource("")
                            .route(web::get().to(bounds))
                        )
                        .service(web::resource("{bound}/stats")
                            .route(web::get().to(bounds_stats))
                        )
                        .service(web::resource("{bound}/meta")
                            .route(web::get().to(bounds_meta))
                        )
                        .service(web::resource("{bound}")
                            .route(web::get().to(bounds_get))
                            .route(web::post().to_async(bounds_set))
                            .route(web::delete().to(bounds_delete))
                        )
                    )
                )
            )
    })
        .workers(workers.unwrap_or(12) as usize)
        .bind(format!("0.0.0.0:{}", port.unwrap_or(8000)).as_str())
        .unwrap()
        .run()
        .unwrap();
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

fn not_found() -> HecateError {
    HecateError::new(404, String::from("Resource Not Found"), None)
}

fn index() -> &'static str { "Hello World!" }

fn server(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth_rules.0.allows_server(&mut auth, &*conn.get()?)?;

    Ok(Json(json!({
        "version": VERSION,
        "constraints": {
            "request": {
                "max_size": MAX_BODY
            }
        }
    })))
}

fn meta_list(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> actix_web::Result<impl Responder> {
    let conn = conn.get()?;

    auth_rules.0.allows_meta_list(&mut auth, &*conn)?;

    let list = serde_json::to_value(meta::list(&*conn)?).unwrap();

    Ok(Json(list))
}


fn meta_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    key: web::Path<String>
) -> actix_web::Result<Json<serde_json::Value>> {
    let conn = conn.get()?;

    auth_rules.0.allows_meta_get(&mut auth, &*conn)?;
    worker.queue(worker::Task::new(worker::TaskType::Meta));

    Ok(Json(meta::Meta::get(&*conn, &key.into_inner())?.value))
}


fn meta_delete(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    key: web::Path<String>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_meta_set(&mut auth, &*conn)?;

    worker.queue(worker::Task::new(worker::TaskType::Meta));

    Ok(Json(json!(meta::delete(&*conn, &key.into_inner())?)))
}

fn meta_set(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    value: Json<serde_json::Value>,
    key: web::Path<String>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_meta_set(&mut auth, &*conn)?;

    worker.queue(worker::Task::new(worker::TaskType::Meta));

    let meta = meta::Meta::new(key.into_inner(), value.into_inner());

    Ok(Json(json!(meta.set(&*conn)?)))
}


fn mvt_get(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    path: web::Path<(u8, u32, u32)>
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;

    let z = path.0;
    let x = path.1;
    let y = path.2;

    auth_rules.0.allows_mvt_get(&mut auth, &*conn)?;

    if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    let tile = mvt::get(&*conn, z, x, y, false)?;

    Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
        .content_type("application/x-protobuf")
        .content_length(tile.len() as u64)
        .body(tile))
}


fn mvt_meta(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    path: web::Path<(u8, u32, u32)>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_mvt_meta(&mut auth, &*conn)?;

    let z = path.0;
    let x = path.1;
    let y = path.2;

    if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    Ok(Json(mvt::meta(&*conn, z, x, y)?))
}

fn mvt_wipe(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_mvt_delete(&mut auth, &*conn)?;

    Ok(Json(mvt::wipe(&*conn)?))
}

fn mvt_regen(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    path: web::Path<(u8, u32, u32)>
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_mvt_regen(&mut auth, &*conn)?;

    let z = path.0;
    let x = path.1;
    let y = path.2;

    if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

    let tile = mvt::get(&*conn, z, x, y, true)?;

    Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
        .content_type("application/x-protobuf")
        .content_length(tile.len() as u64)
        .body(tile))
}

fn user_create(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    user: web::Query<user::User>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_user_create(&mut auth, &*conn)?;

    user.set(&*conn)?;

    worker.queue(worker::Task::new(worker::TaskType::User(user.username.clone())));

    Ok(Json(json!(true)))
}

fn users(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    filter: web::Query<Filter>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_user_list(&mut auth, &*conn)?;

    let filter = filter.into_inner();

    match &filter.filter {
        Some(search) => Ok(Json(json!(user::user::filter(&*conn, &search, &filter.limit)?))),
        None => Ok(Json(user::user::list(&*conn, &filter.limit)?))
    }
}

fn user_info(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.is_admin(&mut auth, &*conn)?;

    let user = user::User::get(&*conn, &uid)?.to_value();

    Ok(Json(user))
}

fn user_set_admin(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.is_admin(&mut auth, &*conn)?;

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
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.is_admin(&mut auth, &*conn)?;

    let mut user = user::User::get(&*conn, &uid)?;

    if !user.is_admin() {
        return Err(HecateError::new(400, format!("{} is not an admin", user.username), None));
    }

    user.admin(false);
    user.set(&*conn)?;

    Ok(Json(json!(true)))
}

fn user_self(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_user_info(&mut auth, &*conn)?;

    let uid = match auth.uid {
        Some(uid) => uid,
        None => { return Err(HecateError::generic(401)); }
    };

    let user = user::User::get(&*conn, &uid)?.to_value();

    Ok(Json(user))

}

fn user_create_session(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_user_create_session(&mut auth, &*conn)?;

    let uid = auth.uid.unwrap();

    let token = user::Token::create(&*conn, "Session Token", &uid)?;

    let cookie = actix_http::http::Cookie::build("session", token.token)
        .path("/")
        .http_only(true)
        .finish();

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK).json(json!(true));
    resp.add_cookie(&cookie).unwrap();

    Ok(resp)
}

fn user_delete_session(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    req: HttpRequest
) -> Result<HttpResponse, HecateError> {
    // there is no auth check here for deleting tokens, the web interface should
    // always be able to de-authenticate to prevent errors

    let token = match req.cookie("session") {
        Some(session) => Some(String::from(session.value())),
        None => None
    };

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK).json(json!(true));
    resp.del_cookie("session");

    match token {
        Some(token) => match auth.uid {
            Some(uid) => match user::token::destroy(&*conn.get()?, &uid, &token) {
                _ => Ok(resp)
            },
            None => Ok(resp)
        },
        None => Ok(resp)
    }
}

fn style_create(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth_rules.0.allows_style_create(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let uid = auth.uid.unwrap();

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };

        let style_id = style::create(&*conn, &uid, &body)?;
        worker.queue(worker::Task::new(worker::TaskType::Style(style_id)));

        Ok(Json(json!(style_id)))
    }))
}

fn style_public(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_style_set_public(&mut auth, &*conn)?;
    let uid = auth.uid.unwrap();

    let style_id = style_id.into_inner();

    Ok(Json(json!(style::access(&*conn, &uid, &style_id, true)?)))
}

fn style_private(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_style_set_private(&mut auth, &*conn)?;
    let uid = auth.uid.unwrap();

    let style_id = style_id.into_inner();

    Ok(Json(json!(style::access(&*conn, &uid, &style_id, false)?)))
}

fn style_patch(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    style_id: web::Path<i64>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    let style_id = style_id.into_inner();

    match auth_rules.0.allows_style_patch(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let uid = auth.uid.unwrap();

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };

        worker.queue(worker::Task::new(worker::TaskType::Style(style_id)));

        Ok(Json(json!(style::update(&*conn, &uid, &style_id, &body)?)))
    }))
}

fn style_delete(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_style_delete(&mut auth, &*conn)?;
    let uid = auth.uid.unwrap();

    let style_id = style_id.into_inner();
    worker.queue(worker::Task::new(worker::TaskType::Style(style_id)));

    Ok(Json(json!(style::delete(&*conn, &uid, &style_id)?)))
}


fn style_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_style_get(&mut auth, &*conn)?;

    let style_id = style_id.into_inner();

    Ok(Json(json!(style::get(&*conn, &auth.uid, &style_id)?)))
}

fn style_list_public(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_style_list(&mut auth, &*conn)?;

    Ok(Json(json!(style::list_public(&*conn)?)))
}

fn style_list_user(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    user_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_style_list(&mut auth, &*conn)?;

    let user_id = user_id.into_inner();

    match auth.uid {
        Some(uid) => {
            if uid == user_id {
                Ok(Json(json!(style::list_user(&*conn, &user_id)?)))
            } else {
                Ok(Json(json!(style::list_user_public(&*conn, &user_id)?)))
            }
        },
        _ => {
            Ok(Json(json!(style::list_user_public(&*conn, &user_id)?)))
        }
    }
}

fn delta_list(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    opts: web::Query<DeltaList>
) ->  Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_delta_list(&mut auth, &*conn)?;

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

fn delta(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) ->  Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_delta_get(&mut auth, &*conn)?;

    Ok(Json(delta::get_json(&*conn, &id.into_inner())?))
}

fn bounds(
    conn: web::Data<DbReplica>,
    mut auth:
    auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    filter: web::Query<Filter>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_bounds_list(&mut auth, &*conn)?;

    let filter = filter.into_inner();
    match filter.filter {
        Some(search) => Ok(Json(json!(bounds::filter(&*conn, &search, &filter.limit)?))),
        None => Ok(Json(json!(bounds::list(&*conn, &filter.limit)?)))
    }
}

fn bounds_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: web::Path<String>
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_bounds_list(&mut auth, &*conn)?;

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
    Ok(resp.streaming(bounds::get(conn, bounds.into_inner())?))
}

fn bounds_set(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: web::Path<String>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth_rules.0.allows_bounds_create(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };
        let geom: serde_json::Value = match serde_json::from_str(&*body) {
            Ok(geom) => geom,
            Err(_) => {
                return Err(HecateError::new(400, String::from("Invalid Feature GeoJSON"), None));
            }
        };

        Ok(Json(json!(bounds::set(&*conn, &bounds, &geom)?)))
    }))
}

fn bounds_delete(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: web::Path<String>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_bounds_delete(&mut auth, &*conn)?;

    Ok(Json(json!(bounds::delete(&*conn, &bounds.into_inner())?)))
}

fn webhooks_list(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_webhooks_list(&mut auth, &*conn)?;

    match serde_json::to_value(webhooks::list(&*conn, webhooks::Action::All)?) {
        Ok(hooks) => Ok(Json(hooks)),
        Err(_) => Err(HecateError::new(500, String::from("Internal Server Error"), None))
    }
}

fn webhooks_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_webhooks_list(&mut auth, &*conn)?;

    match serde_json::to_value(webhooks::get(&*conn, id.into_inner())?) {
        Ok(hooks) => Ok(Json(hooks)),
        Err(_) => Err(HecateError::new(500, String::from("Internal Server Error"), None))
    }
}

fn webhooks_delete(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> Result<Json<bool>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_webhooks_delete(&mut auth, &*conn)?;

    Ok(Json(webhooks::delete(&*conn, id.into_inner())?))
}

fn webhooks_create(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    webhook: Json<webhooks::WebHook>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_webhooks_update(&mut auth, &*conn)?;

    match serde_json::to_value(webhooks::create(&*conn, webhook.into_inner())?) {
        Ok(webhook) => Ok(Json(webhook)),
        Err(_) => { return Err(HecateError::new(500, String::from("Failed to return webhook ID"), None)); }
    }
}

fn webhooks_update(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    mut webhook: Json<webhooks::WebHook>,
    id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_webhooks_update(&mut auth, &*conn)?;

    webhook.id = Some(id.into_inner());

    match serde_json::to_value(webhooks::update(&*conn, webhook.into_inner())?) {
        Ok(webhook) => Ok(Json(webhook)),
        Err(_) => { return Err(HecateError::new(500, String::from("Failed to return webhook ID"), None)); }
    }
}

fn bounds_stats(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bound: web::Path<String>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_stats_bounds(&mut auth, &*conn)?;

    Ok(Json(bounds::stats_json(&*conn, bound.into_inner())?))
}

fn bounds_meta(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bound: web::Path<String>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_bounds_get(&mut auth, &*conn)?;

    Ok(Json(bounds::meta(&*conn, bound.into_inner())?))
}


fn clone_query(
    sandbox_conn: web::Data<DbSandbox>,
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    cquery: web::Query<CloneQuery>
) -> Result<HttpResponse, HecateError> {
    auth_rules.0.allows_clone_query(&mut auth, &*conn.get()?)?;

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
    Ok(resp.streaming(clone::query(sandbox_conn.get()?, &cquery.query, &cquery.limit)?))
}

fn clone_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<HttpResponse, HecateError> {
    auth_rules.0.allows_clone_get(&mut auth, &*conn.get()?)?;

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
    Ok(resp.streaming(clone::get(conn.get()?)?))
}

fn features_query(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    map: web::Query<Map>
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_feature_get(&mut auth, &*conn)?;

    if map.bbox.is_some() && map.point.is_some() {
        Err(HecateError::new(400, String::from("key and point params cannot be used together"), None))
    } else if map.bbox.is_some() {
        let bbox: Vec<f64> = map.bbox.as_ref().unwrap().split(',').map(|s| s.parse().unwrap()).collect();

        let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
        Ok(resp.streaming(feature::get_bbox_stream(conn, &bbox)?))
    } else if map.point.is_some() {
        let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
        Ok(resp.streaming(feature::get_point_stream(conn, &map.point.as_ref().unwrap())?))
    } else {
        Err(HecateError::new(400, String::from("key or point param must be used"), None))
    }

}

fn schema_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    schema: web::Data<Option<serde_json::value::Value>>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_schema_get(&mut auth, &*conn)?;

    match schema.get_ref() {
        Some(s) => Ok(Json(json!(s))),
        None => Err(HecateError::new(404, String::from("No schema Validation Enforced"), None))
    }
}

fn auth_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_auth_get(&mut auth, &*conn)?;

    Ok(Json(auth_rules.0.to_json()))
}

fn stats_get(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_stats_get(&mut auth, &*conn)?;

    Ok(Json(stats::get_json(&*conn)?))
}

fn stats_regen(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;

    auth_rules.0.allows_stats_get(&mut auth, &*conn)?;

    Ok(Json(json!(stats::regen(&*conn)?)))
}

fn features_action(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    worker: web::Data<worker::Worker>,
    schema: web::Data<Option<serde_json::value::Value>>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth_rules.0.allows_feature_create(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let uid = auth.uid.unwrap();

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };


        let mut fc = match body.parse::<GeoJson>() {
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
                        auth_rules.0.allows_feature_force(&mut auth, &*conn)?;
                    }
                }
            };

            match feature::action(&trans, &schema, &feat, &None) {
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
    }))
}

fn osm_map(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    map: web::Query<Map>
) -> Result<String, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_osm_get(&mut auth, &*conn)?;

    let query: Vec<f64> = map.bbox.as_ref().unwrap().split(',').map(|s| s.parse().unwrap()).collect();

    let fc = feature::get_bbox(&*conn, query)?;

    let xml_str = match osm::from_features(&fc) {
        Ok(xml_str) => xml_str,
        Err(err) => { return Err(HecateError::new(417, String::from("Expectation Failed"), Some(err.to_string()))); }
    };

    Ok(xml_str)
}

fn osm_changeset_create(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    body: web::Payload
) -> impl Future<Item = String, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth_rules.0.allows_osm_create(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let uid = auth.uid.unwrap();

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };

        let map = match osm::to_delta(&body) {
            Ok(map) => map,
            Err(err) => { return Err(HecateError::new(500, err.to_string(), None)); }
        };

        let trans = match conn.transaction() {
            Ok(trans) => trans,
            Err(err) => { return Err(HecateError::new(500, String::from("Failed to open transaction"), Some(err.to_string()))); }
        };

        let delta_id = match delta::open(&trans, &map, &uid) {
            Ok(id) => id,
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(err);
            }
        };

        if trans.commit().is_err() {
            return Err(HecateError::new(500, String::from("Failed to commit transaction"), None));
        }

        Ok(delta_id.to_string())
    }))
}

fn osm_changeset_close(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    delta_id: web::Path<i64>
) -> Result<String, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_osm_create(&mut auth, &*conn)?;

    Ok(delta_id.into_inner().to_string())
}

fn osm_changeset_modify(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    delta_id: web::Path<i64>,
    body: web::Payload
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth_rules.0.allows_osm_create(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let uid = auth.uid.unwrap();

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };

        let trans = match conn.transaction() {
            Ok(trans) => trans,
            Err(err) => { return Err(HecateError::new(500, String::from("Failed to open transaction"), Some(err.to_string()))); }
        };

        let delta_id = delta_id.into_inner();

        match delta::is_open(&delta_id, &trans) {
            Ok(true) => (),
            _ => {
                trans.set_rollback();
                trans.finish().unwrap();


                let conflict = format!("The changeset {} was closed at previously", &delta_id);
                return Ok(HttpResponse::build(actix_web::http::StatusCode::CONFLICT)
                    .set_header("Error", conflict.clone())
                    .content_length(conflict.len() as u64)
                    .body(conflict));
            }
        }

        let map = match osm::to_delta(&body) {
            Ok(map) => map,
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(HecateError::new(500, err.to_string(), None));
            }
        };

        let delta_id = match delta::modify_props(&delta_id, &trans, &map, &uid) {
            Ok(id) => id,
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(err);
            }
        };

        if trans.commit().is_err() {
            return Err(HecateError::new(500, String::from("Failed to commit transaction"), None));
        }

        let delta_id = delta_id.to_string();
        Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
            .content_length(delta_id.len() as u64)
            .body(delta_id))
    }))
}

fn osm_changeset_upload(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    schema: web::Data<Option<serde_json::value::Value>>,
    worker: web::Data<worker::Worker>,
    delta_id: web::Path<i64>,
    body: web::Payload
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth_rules.0.allows_osm_create(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let uid = auth.uid.unwrap();

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };

        let trans = match conn.transaction() {
            Ok(trans) => trans,
            Err(_) => { return Err(HecateError::new(500, String::from("Failed to open transaction"), None)); }
        };

        let delta_id = delta_id.into_inner();
        match delta::is_open(&delta_id, &trans) {
            Ok(true) => (),
            _ => {
                trans.set_rollback();
                trans.finish().unwrap();

                let conflict = format!("The changeset {} was closed at previously", &delta_id);
                return Ok(HttpResponse::build(actix_web::http::StatusCode::CONFLICT)
                    .set_header("Error", conflict.clone())
                    .content_length(conflict.len() as u64)
                    .body(conflict));
            }
        }

        let (mut fc, tree) = match osm::to_features(&body) {
            Ok(fctree) => fctree,
            Err(err) => { return Err(HecateError::new(417, err.to_string(), None)); }
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

            let feat_res = match feature::action(&trans, &schema, &feat, &Some(delta_id)) {
                Err(err) => {
                    trans.set_rollback();
                    trans.finish().unwrap();
                    return Err(err);
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
                return Err(HecateError::new(500, String::from("Could not format diffResult XML"), None));
            },
            Ok(diffres) => diffres
        };

        match delta::modify(&delta_id, &trans, &fc, &uid) {
            Ok (_) => (),
            Err(_) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(HecateError::new(500, String::from("Could not create delta"), None));
            }
        }

        match delta::finalize(&delta_id, &trans) {
            Ok (_) => {
                if trans.commit().is_err() {
                    return Err(HecateError::new(500, String::from("Failed to commit transaction"), None));
                }

                worker.queue(worker::Task::new(worker::TaskType::Delta(delta_id)));

                return Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
                    .content_length(diffres.len() as u64)
                    .body(diffres));
            },
            Err(_) => {
                trans.set_rollback();
                trans.finish().unwrap();
                Err(HecateError::new(500, String::from("Could not close delta"), None))
            }
        }
    }))
}

fn osm_capabilities(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<String, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_osm_get(&mut auth, &*conn)?;

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

fn osm_user(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<String, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_osm_get(&mut auth, &*conn)?;

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

fn feature_action(
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    schema: web::Data<Option<serde_json::value::Value>>,
    worker: web::Data<worker::Worker>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth_rules.0.allows_feature_create(&mut auth, &*conn) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let uid = auth.uid.unwrap();

    Either::B(body.map_err(HecateError::from).fold(bytes::BytesMut::new(), move |mut body, chunk| {
        body.extend_from_slice(&chunk);
        Ok::<_, HecateError>(body)
    }).and_then(move |body| {
        let body = match String::from_utf8(body.to_vec()) {
            Ok(body) => body,
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid UTF8 Body"), Some(err.to_string()))); }
        };

        let mut feat = match body.parse::<GeoJson>() {
            Err(_) => { return Err(HecateError::new(400, String::from("Body must be valid GeoJSON Feature"), None)); }
            Ok(geo) => match geo {
                GeoJson::Feature(feat) => feat,
                _ => { return Err(HecateError::new(400, String::from("Body must be valid GeoJSON Feature"), None)); }
            }
        };

        if feature::is_force(&feat)? {
            auth_rules.0.allows_feature_force(&mut auth, &*conn)?;
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

        match feature::action(&trans, &schema, &feat, &None) {
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
    }))
}

fn feature_get(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> Result<HttpResponse, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_feature_get(&mut auth, &*conn)?;

    match feature::get(&*conn, &id.into_inner()) {
        Ok(feature) => {
            let feature = geojson::GeoJson::from(feature).to_string();

            Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
                .content_type("application/json")
                .content_length(feature.len() as u64)
                .body(feature))
        },
        Err(err) => Err(err)
    }
}

fn feature_get_history(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_feature_history(&mut auth, &*conn)?;

    Ok(Json(delta::history(&*conn, &id.into_inner())?))
}

fn feature_query(
    conn: web::Data<DbReplica>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    fquery: web::Query<FeatureQuery>
) -> Result<Json<serde_json::Value>, HecateError> {
    let conn = conn.get()?;
    auth_rules.0.allows_feature_get(&mut auth, &*conn)?;

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
