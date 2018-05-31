#![feature(plugin, custom_derive, custom_attribute, attr_literals)]
#![plugin(rocket_codegen)]

static VERSION: &'static str = "0.29.0";

#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate postgis;
extern crate protobuf;
extern crate rand;
extern crate valico;
extern crate rocket;
extern crate rocket_contrib;
extern crate geojson;
extern crate env_logger;
extern crate chrono;

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

use std::io::{Cursor};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use rocket::http::Status as HTTPStatus;
use rocket::http::{Cookie, Cookies};
use rocket::{Request, State, Outcome};
use rocket::response::{Response, status, Stream, NamedFile};
use rocket::request::{self, FromRequest};
use geojson::GeoJson;
use rocket_contrib::Json;

pub fn start(database: String, schema: Option<serde_json::value::Value>, auth: Option<auth::CustomAuth>) {
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

    rocket::ignite()
        .manage(init_pool(&database))
        .manage(schema)
        .manage(auth_rules)
        .mount("/", routes![
            index
        ])
        .mount("/admin", routes![
            staticsrv
        ])
        .mount("/api", routes![
            meta,
            schema_get,
            auth_get,
            mvt_get,
            mvt_meta,
            mvt_regen,
            user_self,
            user_create,
            user_create_session,
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
            delta_list_params,
            feature_action,
            features_action,
            feature_get,
            feature_query,
            feature_get_history,
            features_get,
            bounds_list,
            bounds_get,
            clone_get,
            xml_capabilities,
            xml_06capabilities,
            xml_user,
            xml_map,
            xml_changeset_create,
            xml_changeset_modify,
            xml_changeset_upload,
            xml_changeset_close
        ])
        .catch(errors![
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

pub struct DbConn(pub r2d2::PooledConnection<PostgresConnectionManager>);
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<PostgresPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((HTTPStatus::ServiceUnavailable, ()))
        }
    }
}

#[error(401)]
fn not_authorized() -> status::Custom<Json> {
    status::Custom(HTTPStatus::Unauthorized, Json(json!({
        "code": 401,
        "status": "Not Authorized",
        "reason": "You must be logged in to access this resource"
    })))
}

#[error(404)]
fn not_found() -> Json {
    Json(json!({
        "code": 404,
        "status": "Not Found",
        "reason": "Resource was not found."
    }))
}


#[get("/")]
fn index() -> &'static str { "Hello World!" }

#[get("/")]
fn meta(mut auth: auth::Auth, conn: DbConn, auth_rules: State<auth::CustomAuth>) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_meta(&mut auth, &conn.0)?;

    Ok(Json(json!({
        "version": VERSION
    })))
}

#[get("/<file..>")]
fn staticsrv(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("web/").join(file)).ok()
}

#[get("/tiles/<z>/<x>/<y>")]
fn mvt_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Response<'static>, status::Custom<String>> {
    auth_rules.allows_mvt_get(&mut auth, &conn.0)?;

    if z > 14 { return Err(status::Custom(HTTPStatus::NotFound, String::from("Tile Not Found"))); }

    let tile = match mvt::get(&conn.0, z, x, y, false) {
        Ok(tile) => tile,
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, err.to_string())); }
    };

    let mut c = Cursor::new(Vec::new());
    match tile.to_writer(&mut c) {
        Ok(_) => (),
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, err.to_string())); }
    }

    let mut mvt_response = Response::new();
    mvt_response.set_status(HTTPStatus::Ok);
    mvt_response.set_sized_body(c);
    mvt_response.set_raw_header("Content-Type", "application/x-protobuf");
    Ok(mvt_response)
}

#[get("/tiles/<z>/<x>/<y>/meta")]
fn mvt_meta(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_mvt_meta(&mut auth, &conn.0)?;

    if z > 14 { return Err(status::Custom(HTTPStatus::NotFound, String::from("Tile Not Found"))); }

    match mvt::meta(&conn.0, z, x, y) {
        Ok(tile) => Ok(Json(tile)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/tiles/<z>/<x>/<y>/regen")]
fn mvt_regen(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, z: u8, x: u32, y: u32) -> Result<Response<'static>, status::Custom<String>> {
    auth_rules.allows_mvt_regen(&mut auth, &conn.0)?;

    if z > 14 { return Err(status::Custom(HTTPStatus::NotFound, String::from("Tile Not Found"))); }

    let tile = match mvt::get(&conn.0, z, x, y, true) {
        Ok(tile) => tile,
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, err.to_string())); }
    };

    let mut c = Cursor::new(Vec::new());
    match tile.to_writer(&mut c) {
        Ok(_) => (),
        Err(err) => { return Err(status::Custom(HTTPStatus::BadRequest, err.to_string())); }
    }

    let mut mvt_response = Response::new();
    mvt_response.set_status(HTTPStatus::Ok);
    mvt_response.set_sized_body(c);
    mvt_response.set_raw_header("Content-Type", "application/x-protobuf");
    Ok(mvt_response)
}

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

#[get("/user/create?<user>")]
fn user_create(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, user: User) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_user_create(&mut auth, &conn.0)?;

    match user::create(&conn.0, &user.username, &user.password, &user.email) {
        Ok(_) => Ok(Json(json!(true))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/user/info")]
fn user_self(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_user_info(&mut auth, &conn.0)?;

    let uid = auth.uid.unwrap();

    match user::info(&conn.0, &uid) {
        Ok(info) => { Ok(Json(json!(info))) },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/user/session")]
fn user_create_session(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, mut cookies: Cookies) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_user_create_session(&mut auth, &conn.0)?;

    let uid = auth.uid.unwrap();

    match user::create_token(&conn.0, &uid) {
        Ok(token) => {
            cookies.add_private(Cookie::new("session", token));
            Ok(Json(json!(uid)))
        },
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[post("/style", format="application/json", data="<style>")]
fn style_create(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, style: String) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_create(&mut auth, &conn.0)?;
    let uid = auth.uid.unwrap();

    match style::create(&conn.0, &uid, &style) {
        Ok(style_id) => Ok(Json(json!(style_id))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[post("/style/<id>/public")]
fn style_public(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_set_public(&mut auth, &conn.0)?;
    let uid = auth.uid.unwrap();

    match style::access(&conn.0, &uid, &id, true) {
        Ok(updated) => Ok(Json(json!(updated))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[post("/style/<id>/private")]
fn style_private(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_set_private(&mut auth, &conn.0)?;
    let uid = auth.uid.unwrap();

    match style::access(&conn.0, &uid, &id, false) {
        Ok(updated) => Ok(Json(json!(updated))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[patch("/style/<id>", format="application/json", data="<style>")]
fn style_patch(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64, style: String) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_patch(&mut auth, &conn.0)?;
    let uid = auth.uid.unwrap();

    match style::update(&conn.0, &uid, &id, &style) {
        Ok(updated) => Ok(Json(json!(updated))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[delete("/style/<id>")]
fn style_delete(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_delete(&mut auth, &conn.0)?;
    let uid = auth.uid.unwrap();

    match style::delete(&conn.0, &uid, &id) {
        Ok(created) => Ok(Json(json!(created))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}


#[get("/style/<id>")]
fn style_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_get(&mut auth, &conn.0)?;

    match style::get(&conn.0, &auth.uid, &id) {
        Ok(style) => Ok(Json(json!(style))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/styles")]
fn style_list_public(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_list(&mut auth, &conn.0)?;

    match style::list_public(&conn.0) {
        Ok(styles) => Ok(Json(json!(styles))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/styles/<user>")]
fn style_list_user(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, user: i64) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_style_list(&mut auth, &conn.0)?;

    match auth.uid {
        Some(uid) => {
            if uid == user {
                match style::list_user(&conn.0, &user) {
                    Ok(styles) => Ok(Json(json!(styles))),
                    Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
                }
            } else {
                match style::list_user_public(&conn.0, &user) {
                    Ok(styles) => Ok(Json(json!(styles))),
                    Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
                }
            }
        },
        _ => {
            match style::list_user_public(&conn.0, &user) {
                Ok(styles) => Ok(Json(json!(styles))),
                Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
            }
        }
    }
}

#[get("/deltas")]
fn delta_list(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) ->  Result<Json, status::Custom<String>> {
    auth_rules.allows_delta_list(&mut auth, &conn.0)?;

    match delta::list_by_offset(&conn.0, None, None) {
        Ok(deltas) => Ok(Json(deltas)),
        Err(err) => Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()))
    }
}

#[derive(FromForm)]
struct DeltaList {
    offset: Option<i64>,
    limit: Option<i64>,
    start: Option<String>,
    end: Option<String>
}

#[get("/deltas?<opts>")]
fn delta_list_params(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, opts: DeltaList) ->  Result<Json, status::Custom<String>> {
    auth_rules.allows_delta_list(&mut auth, &conn.0)?;

    if opts.offset.is_some() && (opts.start.is_some() || opts.end.is_some()) {
        return Err(status::Custom(HTTPStatus::BadRequest, String::from("Offset cannot be used with start or end")));
    }

    if opts.start.is_some() || opts.end.is_some() {
        let start: Option<chrono::NaiveDateTime> = match opts.start {
            None => None,
            Some(start) => {
                match start.parse() {
                    Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Invalid start timestamp"))); },
                    Ok(start) => Some(start)
                }
            }
        };

        let end: Option<chrono::NaiveDateTime> = match opts.end {
            None => None,
            Some(end) => {
                match end.parse() {
                    Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Invalid end timestamp"))); },
                    Ok(end) => Some(end)
                }
            }
        };

        match delta::list_by_date(&conn.0, start, end, opts.limit) {
            Ok(deltas) => {
                return Ok(Json(deltas));
            },
            Err(err) => {
                return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
            }
        }
    } else if opts.offset.is_some() || opts.limit.is_some() {
        match delta::list_by_offset(&conn.0, opts.offset, opts.limit) {
            Ok(deltas) => {
                return Ok(Json(deltas));
            },
            Err(err) => {
                return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
            }
        }
    }
    Err(status::Custom(HTTPStatus::BadRequest, String::from("Query Param Error")))
}

#[get("/delta/<id>")]
fn delta(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) ->  Result<Json, status::Custom<String>> {
    auth_rules.allows_delta_get(&mut auth, &conn.0)?;

    match delta::get_json(&conn.0, &id) {
        Ok(delta) => Ok(Json(delta)),
        Err(err) => Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()))
    }
}

#[get("/data/bounds")]
fn bounds_list(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_bounds_list(&mut auth, &conn.0)?;

    match bounds::list(&conn.0) {
        Ok(bounds) => Ok(Json(json!(bounds))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/data/bounds/<bounds>")]
fn bounds_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, bounds: String) -> Result<Stream<stream::PGStream>, status::Custom<String>> {
    auth_rules.allows_bounds_list(&mut auth, &conn.0)?;

    match bounds::get(conn.0, bounds) {
        Ok(bs) => Ok(Stream::from(bs)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/data/clone")]
fn clone_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Stream<stream::PGStream>, status::Custom<String>> {
    auth_rules.allows_clone_get(&mut auth, &conn.0)?;

    match clone::get(conn.0) {
        Ok(clone) => Ok(Stream::from(clone)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/data/features?<map>")]
fn features_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, map: Map) -> Result<Stream<stream::PGStream>, status::Custom<String>> {
    auth_rules.allows_feature_get(&mut auth, &conn.0)?;

    let bbox: Vec<f64> = map.bbox.split(',').map(|s| s.parse().unwrap()).collect();
    match feature::get_bbox_stream(conn.0, bbox) {
        Ok(features) => Ok(Stream::from(features)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/schema")]
fn schema_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, schema: State<Option<serde_json::value::Value>>) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_schema_get(&mut auth, &conn.0)?;

    match schema.inner().clone() {
        Some(s) => Ok(Json(json!(s.clone()))),
        None => Err(status::Custom(HTTPStatus::NotFound, String::from("No Schema Validation Enforced")))
    }
}

#[get("/auth")]
fn auth_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_auth_get(&mut auth, &conn.0)?;

    Ok(Json(auth_rules.to_json()))
}

#[post("/data/features", format="application/json", data="<body>")]
fn features_action(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: DbConn, schema: State<Option<serde_json::value::Value>>, body: String) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_feature_create(&mut auth, &conn.0)?;

    let uid = auth.uid.unwrap();

    let mut fc = match body.parse::<GeoJson>() {
        Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON Feature"))); },
        Ok(geo) => match geo {
            GeoJson::FeatureCollection(fc) => fc,
            _ => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON FeatureCollection"))); }
        }
    };

    let delta_message = match fc.foreign_members {
        None => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("FeatureCollection Must have message property for delta"))); }
        Some(ref members) => match members.get("message") {
            Some(message) => match message.as_str() {
                Some(message) => String::from(message),
                None => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("FeatureCollection Must have message property for delta"))); }
            },
            None => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("FeatureCollection Must have message property for delta"))); }
        }
    };

    let trans = conn.0.transaction().unwrap();

    let mut map: HashMap<String, Option<String>> = HashMap::new();
    map.insert(String::from("message"), Some(delta_message));

    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not create delta"))); }
    };

    for feat in &mut fc.features {
        match feature::action(&trans, &schema.inner(), &feat, &None) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string()));
            },
            Ok(res) => {
                if res.old == None { feat.id = Some(json!(res.new)); }
            }
        }
    }

    if delta::modify(&delta_id, &trans, &fc, &uid).is_err() {
        trans.set_rollback();
        trans.finish().unwrap();
        return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not create delta")));
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Json(json!(true)))
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()))
        }
    }
}

#[get("/0.6/map?<map>")]
fn xml_map(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, map: Map) -> Result<String, status::Custom<String>> {
    auth_rules.allows_osm_get(&mut auth, &conn.0)?;

    let query: Vec<f64> = map.bbox.split(',').map(|s| s.parse().unwrap()).collect();

    let fc = match feature::get_bbox(&conn.0, query) {
        Ok(features) => features,
        Err(err) => { return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string())) }
    };

    let xml_str = match xml::from_features(&fc) {
        Ok(xml_str) => xml_str,
        Err(err) => { return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string())) }
    };

    Ok(xml_str)
}

#[put("/0.6/changeset/create", data="<body>")]
fn xml_changeset_create(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: DbConn, body: String) -> Result<String, status::Custom<String>> {
    auth_rules.allows_osm_create(&mut auth, &conn.0)?;

    let uid = auth.uid.unwrap();

    let map = match xml::to_delta(&body) {
        Ok(map) => map,
        Err(err) => { return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string())); }
    };

    let trans = conn.0.transaction().unwrap();

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
fn xml_changeset_close(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: DbConn, id: i64) -> Result<String, status::Custom<String>> {
    auth_rules.allows_osm_create(&mut auth, &conn.0)?;

    Ok(id.to_string())
}

#[put("/0.6/changeset/<delta_id>", data="<body>")]
fn xml_changeset_modify(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: DbConn, delta_id: i64, body: String) -> Result<Response<'static>, status::Custom<String>> {
    auth_rules.allows_osm_create(&mut auth, &conn.0)?;

    let uid = auth.uid.unwrap();

    let trans = conn.0.transaction().unwrap();

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
fn xml_changeset_upload(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: DbConn, schema: State<Option<serde_json::value::Value>>, delta_id: i64, body: String) -> Result<Response<'static>, status::Custom<String>> {
    auth_rules.allows_osm_create(&mut auth, &conn.0)?;

    let uid = auth.uid.unwrap();

    let trans = conn.0.transaction().unwrap();

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
        let feat_res = match feature::action(&trans, &schema.inner(), &feat, &Some(delta_id)) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string()));
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
fn xml_capabilities(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
    auth_rules.allows_osm_get(&mut auth, &conn.0)?;

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
fn xml_06capabilities(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
    auth_rules.allows_osm_get(&mut auth, &conn.0)?;

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
fn xml_user(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>) -> Result<String, status::Custom<String>> {
    auth_rules.allows_osm_get(&mut auth, &conn.0)?;

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
fn feature_action(mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, conn: DbConn, schema: State<Option<serde_json::value::Value>>, body: String) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_feature_create(&mut auth, &conn.0)?;

    let uid = auth.uid.unwrap();

    let mut feat = match body.parse::<GeoJson>() {
        Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON Feature"))); },
        Ok(geo) => match geo {
            GeoJson::Feature(feat) => feat,
            _ => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON Feature"))); }
        }
    };

    let delta_message = match feat.foreign_members {
        None => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Feature Must have message property for delta"))); }
        Some(ref members) => match members.get("message") {
            Some(message) => match message.as_str() {
                Some(message) => String::from(message),
                None => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Feature Must have message property for delta"))); }
            },
            None => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Feature Must have message property for delta"))); }
        }
    };

    let trans = conn.0.transaction().unwrap();

    let mut map: HashMap<String, Option<String>> = HashMap::new();
    map.insert(String::from("message"), Some(delta_message));
    let delta_id = match delta::open(&trans, &map, &uid) {
        Ok(id) => id,
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not create delta")));
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
            return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string()));
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
        return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not create delta")));
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Json(json!(true)))
        },
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()))
        }
    }
}

#[get("/data/feature/<id>")]
fn feature_get(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<String, status::Custom<String>> {
    auth_rules.allows_feature_get(&mut auth, &conn.0)?;

    match feature::get(&conn.0, &id) {
        Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[derive(FromForm)]
struct FeatureQuery {
    key: Option<String>
}

#[get("/data/feature?<fquery>")]
fn feature_query(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, fquery: FeatureQuery) -> Result<String, status::Custom<String>> {
    auth_rules.allows_feature_get(&mut auth, &conn.0)?;

    if fquery.key.is_some() {
        match feature::query_by_key(&conn.0, &fquery.key.unwrap()) {
            Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
            Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
        }
    } else {
        Err(status::Custom(HTTPStatus::BadRequest, String::from("At least 1 query parameter must be specified")))
    }
}

#[get("/data/feature/<id>/history")]
fn feature_get_history(conn: DbConn, mut auth: auth::Auth, auth_rules: State<auth::CustomAuth>, id: i64) -> Result<Json, status::Custom<String>> {
    auth_rules.allows_feature_history(&mut auth, &conn.0)?;

    match delta::history(&conn.0, &id) {
        Ok(features) => Ok(Json(features)),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}
