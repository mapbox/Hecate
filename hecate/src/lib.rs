pub static VERSION: &'static str = "0.82.1";
pub static POSTGRES: f64 = 10.0;
pub static POSTGIS: f64 = 2.4;
pub static HOURS: i64 = 24;

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

use actix_http::error::ResponseError;
use actix_http::httpmessage::HttpMessage;
use actix_web::{web, web::Json, App, HttpResponse, HttpRequest, HttpServer, middleware};
use futures::{Future, Stream, future::Either};
use geojson::GeoJson;
use crate::{
    auth::AuthModule,
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
    let auth_rules: auth::CustomAuth = match auth {
        None => auth::CustomAuth::default(),
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

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let default = match &*auth_rules.0.default {
        "public"  => auth::ServerAuthDefault::Public,
        "user" => auth::ServerAuthDefault::User,
        "admin" => auth::ServerAuthDefault::Admin,
        _ => panic!("Invalid 'default' value in custom auth")
    };

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Logger::default())
            .wrap(auth::middleware::EnforceAuth::new(db_replica.clone(), default.clone()))
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
                actix_files::Files::new("/admin", "../hecate_ui/dist/")
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
                    .route(web::get().to_async(meta_list))
                )
                .service(web::resource("meta/{key}")
                    .route(web::post().to_async(meta_set))
                    .route(web::delete().to_async(meta_delete))
                    .route(web::get().to_async(meta_get))
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
                    .route(web::get().to_async(delta_list))
                )
                .service(web::resource("delta/{id}")
                    .route(web::get().to_async(delta))
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
                        .route(web::delete().to_async(mvt_wipe))
                    )
                    .service(web::resource("{z}/{x}/{y}")
                        .route(web::get().to_async(mvt_get))
                    )
                    .service(web::resource("{z}/{x}/{y}/meta")
                        .route(web::get().to_async(mvt_meta))
                    )
                    .service(web::resource("{z}/{x}/{y}/regen")
                        .route(web::get().to_async(mvt_regen))
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
                    .service(web::resource("reset")
                        .route(web::post().to(user_pwreset))
                    )
                    .service(web::resource("session")
                        .route(web::get().to_async(user_create_session))
                        .route(web::delete().to_async(user_delete_session))
                    )
                    .service(web::resource("token")
                        .route(web::post().to(user_create_token))
                    )
                    .service(web::resource("token/{token}")
                        .route(web::delete().to(user_delete_token))
                    )
                    .service(web::resource("{uid}")
                        .route(web::get().to(user_info))
                        .route(web::post().to_async(user_modify_info))
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
                        .route(web::get().to_async(feature_get_history))
                    )
                    .service(web::resource("features")
                        .route(web::post().to_async(features_action))
                        .route(web::get().to(features_query))
                    )
                    .service(web::resource("features/history")
                        .route(web::get().to(features_history_query))
                    )
                    .service(web::resource("stats")
                        .route(web::get().to_async(stats_get))
                    )
                    .service(web::resource("stats/regen")
                        .route(web::get().to_async(stats_regen))
                    )
                    .service(web::resource("query")
                        .route(web::get().to(clone_query))
                    )
                    .service(web::resource("clone")
                        .route(web::get().to(clone_get))
                    )
                    .service(web::scope("bounds")
                        .service(web::resource("")
                            .route(web::get().to_async(bounds))
                        )
                        .service(web::resource("{bound}/stats")
                            .route(web::get().to_async(bounds_stats))
                        )
                        .service(web::resource("{bound}/meta")
                            .route(web::get().to_async(bounds_meta))
                        )
                        .service(web::resource("{bound}")
                            .route(web::get().to(bounds_get))
                            .route(web::post().to_async(bounds_set))
                            .route(web::delete().to_async(bounds_delete))
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
struct PwReset {
    current: String,
    update: String
}

#[derive(Deserialize, Debug)]
struct Map {
    bbox: Option<String>,
    point: Option<String>
}

#[derive(Deserialize, Debug)]
struct Token {
    name: Option<String>,
    hours: Option<i64>,
    scope: Option<String> //read, full (default read)
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

fn index() -> &'static str { "Hello World!" }

fn server(
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.server, auth::RW::Read, &auth)?;

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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.meta.get, auth::RW::Read, &auth)?;

        Ok(serde_json::to_value(meta::list(&*conn.get()?)?).unwrap())
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(list) => Ok(actix_web::HttpResponse::Ok().json(list)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}


fn meta_get(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    key: web::Path<String>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.meta.get, auth::RW::Read, &auth)?;

        worker.queue(worker::Task::new(worker::TaskType::Meta));

        Ok(meta::Meta::get(&*conn.get()?, &key.into_inner())?.value)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(meta) => Ok(actix_web::HttpResponse::Ok().json(meta)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}


fn meta_delete(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    key: web::Path<String>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.meta.set, auth::RW::Full, &auth)?;

        worker.queue(worker::Task::new(worker::TaskType::Meta));

        Ok(json!(meta::delete(&*conn.get()?, &key.into_inner())?))
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(meta) => Ok(actix_web::HttpResponse::Ok().json(meta)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn meta_set(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    value: Json<serde_json::Value>,
    key: web::Path<String>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.meta.set, auth::RW::Full, &auth)?;

        worker.queue(worker::Task::new(worker::TaskType::Meta));

        let meta = meta::Meta::new(key.into_inner(), value.into_inner());

        Ok(json!(meta.set(&*conn.get()?)?))
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(meta) => Ok(actix_web::HttpResponse::Ok().json(meta)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}


fn mvt_get(
    conn_write: web::Data<DbReadWrite>,
    conn_read: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    path: web::Path<(u8, u32, u32)>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.mvt.get, auth::RW::Read, &auth)?;

        let z = path.0;
        let x = path.1;
        let y = path.2;

        if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

        Ok(mvt::get(&*conn_read.get()?, &*conn_write.get()?, z, x, y, false)?)
    }).then(|res: Result<Vec<u8>, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(tile) => {
            Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
                .content_type("application/x-protobuf")
                .content_length(tile.len() as u64)
                .body(tile))
        },
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}


fn mvt_meta(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    path: web::Path<(u8, u32, u32)>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.mvt.meta, auth::RW::Read, &auth)?;

        let z = path.0;
        let x = path.1;
        let y = path.2;

        if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

        Ok(mvt::meta(&*conn.get()?, z, x, y)?)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(meta) => Ok(actix_web::HttpResponse::Ok().json(meta)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn mvt_wipe(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.mvt.delete, auth::RW::Full, &auth)?;

        Ok(mvt::wipe(&*conn.get()?)?)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(wipe) => Ok(actix_web::HttpResponse::Ok().json(wipe)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn mvt_regen(
    conn_write: web::Data<DbReadWrite>,
    conn_read: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    path: web::Path<(u8, u32, u32)>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.mvt.regen, auth::RW::Full, &auth)?;

        let z = path.0;
        let x = path.1;
        let y = path.2;

        if z > 17 { return Err(HecateError::new(404, String::from("Tile Not Found"), None)); }

        Ok(mvt::get(&*conn_read.get()?, &*conn_write.get()?, z, x, y, true)?)
    }).then(|res: Result<Vec<u8>, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(tile) => {
            Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
                .content_type("application/x-protobuf")
                .content_length(tile.len() as u64)
                .body(tile))
        },
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn user_create(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    user: web::Query<user::User>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.user.create, auth::RW::Full, &auth)?;

    user.set(&*conn.get()?)?;

    worker.queue(worker::Task::new(worker::TaskType::User(user.username.clone())));

    Ok(Json(json!(true)))
}

fn users(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    filter: web::Query<Filter>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.user.list, auth::RW::Read, &auth)?;

    let filter = filter.into_inner();

    match &filter.filter {
        Some(search) => Ok(Json(json!(user::user::filter(&*conn.get()?, &search, &filter.limit)?))),
        None => Ok(Json(user::user::list(&*conn.get()?, &filter.limit)?))
    }
}

fn user_info(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth_rules.0.is_admin(&auth)?;

    let user = user::User::get(&*conn.get()?, &uid)?.to_value();

    Ok(Json(user))
}

fn user_modify_info(
    conn: web::Data<DbReadWrite>,
    mut auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: web::Path<i64>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    match auth_rules.0.is_admin(&mut auth) {
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

        let mut user: user::User = match serde_json::from_str(&*body) {
            Ok(user) => match serde_json::from_value(user) {
                Ok(user) => user,
                Err(err) => { return Err(HecateError::new(500, String::from("Failed to deserialize user"), Some(err.to_string()))); }
            },
            Err(err) => { return Err(HecateError::new(400, String::from("Invalid JSON"), Some(err.to_string()))); }
        };

        user.id = Some(*uid);

        Ok(Json(json!(user.set(&*conn.get()?)?)))
    }))
}

fn user_set_admin(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth_rules.0.is_admin(&auth)?;

    let conn = conn.get()?;

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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    uid: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth_rules.0.is_admin(&auth)?;

    let conn = conn.get()?;

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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.user.info, auth::RW::Read, &auth)?;

    let uid = match auth.uid {
        Some(uid) => uid,
        None => { return Err(HecateError::generic(401)); }
    };

    let user = user::User::get(&*conn.get()?, &uid)?.to_value();

    Ok(Json(user))

}

fn user_pwreset(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    reset: Json<PwReset>
) -> Result<Json<serde_json::Value>, HecateError> {
    // No auth rules here - user can always change their password

    let uid = match auth.uid {
        Some(uid) => uid,
        None => { return Err(HecateError::generic(401)); }
    };

    user::User::reset(&*conn.get()?, &uid, &reset.current, &reset.update)?;

    Ok(Json(json!(true)))

}

fn user_create_session(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.user.create_session, auth::RW::Full, &auth)?;

        let uid = auth.uid.unwrap();

        Ok(user::Token::create(&*conn.get()?, "Session Token", &uid, &HOURS, user::token::Scope::Full)?)
    }).then(|res: Result<user::Token, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(token) => {

            let cookie = actix_http::http::Cookie::build("session", token.token)
                .path("/")
                .http_only(true)
                .max_age(HOURS * 60 * 60)
                .finish();

            let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK).json(json!(true));
            resp.add_cookie(&cookie).unwrap();

            Ok(resp)
        },
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn user_delete_session(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    req: HttpRequest
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    let token = match req.cookie("session") {
        Some(session) => Some(String::from(session.value())),
        None => None
    };

    web::block(move || {
        // there is no auth check here for deleting tokens, the web interface should
        // always be able to de-authenticate to prevent errors
        match token {
            Some(token) => match auth.uid {
                Some(uid) => match user::token::destroy(&*conn.get()?, &uid, &token) {
                    _ => Ok(true)
                },
                None => Ok(true)
            },
            None => Ok(true)
        }
    }).then(|res: Result<bool, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(_) => {
            let cookie = actix_http::http::Cookie::build("session", String::from(""))
                .path("/")
                .http_only(true)
                .finish();

            let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK).json(json!(true));
            resp.add_cookie(&cookie).unwrap();

            Ok(resp)
        },
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn user_create_token(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    token: web::Query<Token>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.user.create_session, auth::RW::Full, &auth)?;

    let token = token.into_inner();

    let uid = auth.uid.unwrap();

    let scope = match token.scope {
        Some(scope) => match scope.as_str() {
            "full" => user::token::Scope::Full,
            "read" => user::token::Scope::Read,
            _ => {
                return Err(HecateError::new(400, String::from("Invalid Token Scope"), None));
            },
        },
        None => user::token::Scope::Read
    };

    let token = user::Token::create(
        &*conn.get()?,
        token.name.unwrap_or(String::from("Access Token")),
        &uid,
        &token.hours.unwrap_or(16),
        scope
    )?;

    match serde_json::to_value(token) {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(HecateError::new(500, String::from("Internal Server Error"), None))
    }
}

fn user_delete_token(
    conn: web::Data<DbReadWrite>,
    auth_rules: web::Data<auth::AuthContainer>,
    auth: auth::Auth,
    token: web::Path<String>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.user.create_session, auth::RW::Full, &auth)?;

    let uid = auth.uid.unwrap();

    let token = token.into_inner();

    user::token::destroy(&*conn.get()?, &uid, &token)?;

    Ok(Json(json!(true)))
}

fn style_create(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth::check(&auth_rules.0.style.create, auth::RW::Full, &auth) {
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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.style.set_public, auth::RW::Full, &auth)?;
    let uid = auth.uid.unwrap();

    let style_id = style_id.into_inner();

    Ok(Json(json!(style::access(&*conn.get()?, &uid, &style_id, true)?)))
}

fn style_private(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.style.set_private, auth::RW::Full, &auth)?;
    let uid = auth.uid.unwrap();

    let style_id = style_id.into_inner();

    Ok(Json(json!(style::access(&*conn.get()?, &uid, &style_id, false)?)))
}

fn style_patch(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
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

    match auth::check(&auth_rules.0.style.patch, auth::RW::Full, &auth) {
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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    worker: web::Data<worker::Worker>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.style.delete, auth::RW::Full, &auth)?;
    let uid = auth.uid.unwrap();

    let style_id = style_id.into_inner();
    worker.queue(worker::Task::new(worker::TaskType::Style(style_id)));

    Ok(Json(json!(style::delete(&*conn.get()?, &uid, &style_id)?)))
}


fn style_get(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    style_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.style.get, auth::RW::Read, &auth)?;

    let style_id = style_id.into_inner();

    Ok(Json(json!(style::get(&*conn.get()?, &auth.uid, &style_id)?)))
}

fn style_list_public(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.style.list, auth::RW::Read, &auth)?;

    Ok(Json(json!(style::list_public(&*conn.get()?)?)))
}

fn style_list_user(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    user_id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.style.list, auth::RW::Read, &auth)?;

    let user_id = user_id.into_inner();

    match auth.uid {
        Some(uid) => {
            if uid == user_id {
                Ok(Json(json!(style::list_user(&*conn.get()?, &user_id)?)))
            } else {
                Ok(Json(json!(style::list_user_public(&*conn.get()?, &user_id)?)))
            }
        },
        _ => {
            Ok(Json(json!(style::list_user_public(&*conn.get()?, &user_id)?)))
        }
    }
}

fn delta_list(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    opts: web::Query<DeltaList>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.delta.list, auth::RW::Read, &auth)?;

        if opts.offset.is_none() && opts.limit.is_none() && opts.start.is_none() && opts.end.is_none() {
            Ok(delta::list_by_offset(&*conn.get()?, None, None)?)
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

            Ok(delta::list_by_date(&*conn.get()?, start, end, opts.limit)?)
        } else if opts.offset.is_some() || opts.limit.is_some() {
            Ok(delta::list_by_offset(&*conn.get()?, opts.offset, opts.limit)?)
        } else {
            return Err(HecateError::new(400, String::from("Invalid Query Params"), None));
        }
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(list) => Ok(actix_web::HttpResponse::Ok().json(list)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn delta(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.delta.get, auth::RW::Read, &auth)?;

        Ok(delta::get_json(&*conn.get()?, &id.into_inner())?)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(delta) => Ok(actix_web::HttpResponse::Ok().json(delta)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn bounds(
    conn: web::Data<DbReplica>,
    auth:
    auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    filter: web::Query<Filter>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.bounds.list, auth::RW::Read, &auth)?;

        let filter = filter.into_inner();
        match filter.filter {
            Some(search) => Ok(json!(bounds::filter(&*conn.get()?, &search, &filter.limit)?)),
            None => Ok(json!(bounds::list(&*conn.get()?, &filter.limit)?))
        }
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(bounds) => Ok(actix_web::HttpResponse::Ok().json(bounds)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn bounds_get(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: web::Path<String>
) -> Result<HttpResponse, HecateError> {
    auth::check(&auth_rules.0.bounds.list, auth::RW::Read, &auth)?;

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
    Ok(resp.streaming(bounds::get(conn.get()?, bounds.into_inner())?))
}

fn bounds_set(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: web::Path<String>,
    body: web::Payload
) -> impl Future<Item = Json<serde_json::Value>, Error = HecateError> {
    match auth::check(&auth_rules.0.bounds.create, auth::RW::Full, &auth) {
        Err(err) => { return Either::A(futures::future::err(err)); },
        _ => ()
    };

    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bounds: web::Path<String>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.bounds.delete, auth::RW::Full, &auth)?;

        Ok(json!(bounds::delete(&*conn.get()?, &bounds.into_inner())?))
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(bounds) => Ok(actix_web::HttpResponse::Ok().json(bounds)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn webhooks_list(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.webhooks.get, auth::RW::Read, &auth)?;

    let hooks = webhooks::list(&*conn.get()?, webhooks::Action::All)?;
    let values: Vec<serde_json::Value> = hooks.into_iter().map(|h| h.to_value()).collect();
    Ok(Json(serde_json::Value::Array(values)))
}

fn webhooks_get(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.webhooks.get, auth::RW::Read, &auth)?;

    let hook = webhooks::get(&*conn.get()?, id.into_inner())?.to_value();
    Ok(Json(hook))
}

fn webhooks_delete(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> Result<Json<bool>, HecateError> {
    auth::check(&auth_rules.0.webhooks.set, auth::RW::Full, &auth)?;

    Ok(Json(webhooks::delete(&*conn.get()?, id.into_inner())?))
}

fn webhooks_create(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    webhook: Json<webhooks::WebHook>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.webhooks.set, auth::RW::Full, &auth)?;

    match serde_json::to_value(webhooks::create(&*conn.get()?, webhook.into_inner())?) {
        Ok(webhook) => Ok(Json(webhook)),
        Err(_) => { return Err(HecateError::new(500, String::from("Failed to create webhook"), None)); }
    }
}

fn webhooks_update(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    mut webhook: Json<webhooks::WebHook>,
    id: web::Path<i64>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.webhooks.set, auth::RW::Full, &auth)?;

    webhook.id = Some(id.into_inner());

    let hook = webhooks::update(&*conn.get()?, webhook.into_inner())?.to_value();
    Ok(Json(hook))
}

fn bounds_stats(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bound: web::Path<String>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.stats.get, auth::RW::Read, &auth)?;

        Ok(bounds::stats_json(&*conn.get()?, bound.into_inner())?)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(stats) => Ok(actix_web::HttpResponse::Ok().json(stats)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn bounds_meta(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    bound: web::Path<String>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.bounds.get, auth::RW::Read, &auth)?;

        Ok(bounds::meta(&*conn.get()?, bound.into_inner())?)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(stats) => Ok(actix_web::HttpResponse::Ok().json(stats)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn clone_query(
    sandbox_conn: web::Data<DbSandbox>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    cquery: web::Query<CloneQuery>
) -> Result<HttpResponse, HecateError> {
    auth::check(&auth_rules.0.clone.query, auth::RW::Read, &auth)?;

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
    Ok(resp.streaming(clone::query(sandbox_conn.get()?, &cquery.query, &cquery.limit)?))
}

fn clone_get(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<HttpResponse, HecateError> {
    auth::check(&auth_rules.0.clone.get, auth::RW::Read, &auth)?;

    let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
    Ok(resp.streaming(clone::get(conn.get()?)?))
}

fn features_query(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    map: web::Query<Map>
) -> Result<HttpResponse, HecateError> {
    auth::check(&auth_rules.0.feature.get, auth::RW::Read, &auth)?;

    if map.bbox.is_some() && map.point.is_some() {
        Err(HecateError::new(400, String::from("key and point params cannot be used together"), None))
    } else if map.bbox.is_some() {
        let bbox: Vec<f64> = map.bbox.as_ref().unwrap().split(',').map(|s| s.parse().unwrap()).collect();

        let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
        Ok(resp.streaming(feature::get_bbox_stream(conn.get()?, &bbox)?))
    } else if map.point.is_some() {
        let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
        Ok(resp.streaming(feature::get_point_stream(conn.get()?, &map.point.as_ref().unwrap())?))
    } else {
        Err(HecateError::new(400, String::from("key or point param must be used"), None))
    }
}

fn features_history_query(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    map: web::Query<Map>
) -> Result<HttpResponse, HecateError> {
    auth::check(&auth_rules.0.feature.history, auth::RW::Read, &auth)?;

    if map.bbox.is_some() && map.point.is_some() {
        Err(HecateError::new(400, String::from("key and point params cannot be used together"), None))
    } else if map.bbox.is_some() {
        let bbox: Vec<f64> = map.bbox.as_ref().unwrap().split(',').map(|s| s.parse().unwrap()).collect();

        let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
        Ok(resp.streaming(feature::get_bbox_history_stream(conn.get()?, &bbox)?))
    } else if map.point.is_some() {
        let mut resp = HttpResponse::build(actix_web::http::StatusCode::OK);
        Ok(resp.streaming(feature::get_point_history_stream(conn.get()?, &map.point.as_ref().unwrap())?))
    } else {
        Err(HecateError::new(400, String::from("key or point param must be used"), None))
    }
}

fn schema_get(
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    schema: web::Data<Option<serde_json::value::Value>>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.schema.get, auth::RW::Read, &auth)?;

    match schema.get_ref() {
        Some(s) => Ok(Json(json!(s))),
        None => Err(HecateError::new(404, String::from("No schema Validation Enforced"), None))
    }
}

fn auth_get(
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.auth.get, auth::RW::Read, &auth)?;

    Ok(Json(auth_rules.0.to_json()?))
}

fn stats_get(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.stats.get, auth::RW::Read, &auth)?;

        Ok(stats::get_json(&*conn.get()?)?)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(stats) => Ok(actix_web::HttpResponse::Ok().json(stats)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn stats_regen(
    conn: web::Data<DbReadWrite>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.stats.get, auth::RW::Read, &auth)?;

        Ok(json!(stats::regen(&*conn.get()?)?))
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(stats) => Ok(actix_web::HttpResponse::Ok().json(stats)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn features_action(
    auth: auth::Auth,
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

    match auth::check(&auth_rules.0.feature.create, auth::RW::Full, &auth) {
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
                        auth::check(&auth_rules.0.feature.force, auth::RW::Full, &auth)?;
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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    map: web::Query<Map>
) -> Result<String, HecateError> {
    auth::check(&auth_rules.0.osm.get, auth::RW::Read, &auth)?;

    let query: Vec<f64> = map.bbox.as_ref().unwrap().split(',').map(|s| s.parse().unwrap()).collect();

    let fc = feature::get_bbox(&*conn.get()?, query)?;

    let xml_str = match osm::from_features(&fc) {
        Ok(xml_str) => xml_str,
        Err(err) => { return Err(HecateError::new(417, String::from("Expectation Failed"), Some(err.to_string()))); }
    };

    Ok(xml_str)
}

fn osm_changeset_create(
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    body: web::Payload
) -> impl Future<Item = String, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth::check(&auth_rules.0.osm.create, auth::RW::Full, &auth) {
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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    delta_id: web::Path<i64>
) -> Result<String, HecateError> {
    auth::check(&auth_rules.0.osm.create, auth::RW::Full, &auth)?;

    Ok(delta_id.into_inner().to_string())
}

fn osm_changeset_modify(
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    conn: web::Data<DbReadWrite>,
    delta_id: web::Path<i64>,
    body: web::Payload
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    let conn = match conn.get() {
        Ok(conn) => conn,
        Err(err) => { return Either::A(futures::future::err(err)); }
    };

    match auth::check(&auth_rules.0.osm.create, auth::RW::Full, &auth) {
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
    auth: auth::Auth,
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

    match auth::check(&auth_rules.0.osm.create, auth::RW::Full, &auth) {
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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<String, HecateError> {
    auth::check(&auth_rules.0.osm.get, auth::RW::Read, &auth)?;

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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>
) -> Result<String, HecateError> {
    auth::check(&auth_rules.0.osm.get, auth::RW::Read, &auth)?;

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
    auth: auth::Auth,
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

    match auth::check(&auth_rules.0.feature.create, auth::RW::Full, &auth) {
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
            auth::check(&auth_rules.0.feature.force, auth::RW::Full, &auth)?;
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
        let delta_id = match delta::open(&trans, &map, &uid) { // add non feature info to deltas table, get next delta id
            Ok(id) => id,
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(err);
            }
        };
        // inserts feature into geo table
        // version is incremented by 1 here
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
        // modifies the delta entry to include the features json blob
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
                // triggers webhook
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
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.feature.get, auth::RW::Read, &auth)?;

        match feature::get(&*conn.get()?, &id.into_inner()) {
            Ok(feature) => Ok(geojson::GeoJson::from(feature).to_string()),
            Err(err) => Err(err)
        }
    }).then(|res: Result<String, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(feature) => {
            Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
                .content_type("application/json")
                .content_length(feature.len() as u64)
                .body(feature))
        },
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn feature_get_history(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    id: web::Path<i64>
) -> impl Future<Item = HttpResponse, Error = HecateError> {
    web::block(move || {
        auth::check(&auth_rules.0.feature.history, auth::RW::Read, &auth)?;

        Ok(feature::history(&*conn.get()?, &id.into_inner())?)
    }).then(|res: Result<serde_json::Value, actix_threadpool::BlockingError<HecateError>>| match res {
        Ok(history) => Ok(actix_web::HttpResponse::Ok().json(history)),
        Err(err) => Ok(HecateError::from(err).error_response())
    })
}

fn feature_query(
    conn: web::Data<DbReplica>,
    auth: auth::Auth,
    auth_rules: web::Data<auth::AuthContainer>,
    fquery: web::Query<FeatureQuery>
) -> Result<Json<serde_json::Value>, HecateError> {
    auth::check(&auth_rules.0.feature.get, auth::RW::Read, &auth)?;

    if fquery.key.is_some() && fquery.point.is_some() {
        Err(HecateError::new(400, String::from("key and point params cannot be used together"), None))
    } else if fquery.key.is_some() {
        Ok(Json(feature::query_by_key(&*conn.get()?, &fquery.key.as_ref().unwrap())?))
    } else if fquery.point.is_some() {
        let mut results = feature::query_by_point(&*conn.get()?, &fquery.point.as_ref().unwrap())?;
        Ok(Json(results.pop().unwrap()))
    } else {
        Err(HecateError::new(400, String::from("key or point param must be used"), None))
    }
}
