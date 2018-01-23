#![feature(plugin, custom_derive, custom_attribute, attr_literals)]
#![plugin(rocket_codegen)]

extern crate hecate;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate clap;
#[macro_use] extern crate serde_json;
extern crate geojson;
extern crate base64;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate env_logger;
extern crate tempdir;
extern crate fallible_iterator;

//Postgres Connection Pooling
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use mvt::Encode;

use rocket_contrib::Json as Json;
use std::io::{Write, Cursor};
use std::path::{Path, PathBuf};
use std::fs::File;
use tempdir::TempDir;
use std::collections::HashMap;
use rocket::http::Status as HTTPStatus;
use rocket::{Request, State, Outcome};
use rocket::response::{Response, status, Stream, NamedFile};
use rocket::request::{self, FromRequest};
use clap::App;
use geojson::GeoJson;
use hecate::{feature, user, bounds, delta, xml, mvt};
use fallible_iterator::FallibleIterator;

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

struct HTTPAuth {
    username: String,
    password: String
}
impl<'a, 'r> FromRequest<'a, 'r> for HTTPAuth {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<HTTPAuth, ()> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();

        if keys.len() != 1 || keys[0].len() < 7 { return Outcome::Failure((HTTPStatus::Unauthorized, ())); }

        let mut authtype = String::from(keys[0]);
        let auth = authtype.split_off(6);

        if authtype != "Basic " { return Outcome::Failure((HTTPStatus::Unauthorized, ())); }

        match base64::decode(&auth) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(decoded_str) => {
                    let split = decoded_str.split(":").collect::<Vec<&str>>();

                    if split.len() != 2 { return Outcome::Failure((HTTPStatus::Unauthorized, ())); }

                    Outcome::Success(HTTPAuth {
                        username: String::from(split[0]),
                        password: String::from(split[1])
                    })
                },
                Err(_) => Outcome::Failure((HTTPStatus::Unauthorized, ()))
            },
            Err(_) => Outcome::Failure((HTTPStatus::Unauthorized, ()))
        }
    }
}

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let matched = App::from_yaml(cli_cnf).get_matches();

    let database = matched.value_of("database").unwrap_or("postgres@localhost:5432/hecate");

    env_logger::init();

    rocket::ignite()
        .manage(init_pool(&database))
        .mount("/", routes![
            index
        ])
        .mount("/admin", routes![
            staticsrv
        ])
        .mount("/api", routes![
            mvt_get,
            user_create,
            delta_list,
            delta_list_offset,
            feature_action,
            features_action,
            feature_get,
            features_get,
            bounds_list,
            bounds_get,
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

#[get("/")]
fn index() -> &'static str { "Hello World!" }

#[get("/<file..>")]
fn staticsrv(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("web/").join(file)).ok()
}

#[get("/tiles/<z>/<x>/<y>")]
fn mvt_get(conn: DbConn, z: u8, x: u32, y: u32) -> Result<Response<'static>, status::Custom<String>> {
    let tile = match mvt::get(&conn.0, z, x, y) {
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

#[derive(FromForm)]
struct DeltaList {
    offset: i64
}

#[get("/user/create?<user>")]
fn user_create(conn: DbConn, user: User) -> Result<Json, status::Custom<String>> {
    match user::create(&conn.0, &user.username, &user.password, &user.email) {
        Ok(_) => Ok(Json(json!(true))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/deltas")]
fn delta_list(conn: DbConn) ->  Result<Json, status::Custom<String>> {
    match delta::list_json(&conn.0, None) {
        Ok(deltas) => Ok(deltas),
        Err(err) => Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()))
    }
}

#[get("/deltas?<opts>")]
fn delta_list_offset(conn: DbConn, opts: DeltaList) ->  Result<Json, status::Custom<String>> {
    match delta::list_json(&conn.0, Some(opts.offset)) {
        Ok(deltas) => Ok(deltas),
        Err(err) => Err(status::Custom(HTTPStatus::InternalServerError, err.to_string()))
    }
}

#[get("/data/bounds")]
fn bounds_list(conn: DbConn) -> Result<Json, status::Custom<String>> {
    match bounds::list(&conn.0) {
        Ok(bounds) => Ok(Json(json!(bounds))),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[get("/data/bounds/<bounds>")]
fn bounds_get(conn: DbConn, bounds: String) -> Result<Stream<std::fs::File>, status::Custom<String>> {
    let trans = conn.0.transaction().unwrap();

    let query = bounds::get_query();

    let stmt = match trans.prepare(&query) {
        Ok(stmt) => stmt,
        Err(err) => {
            match err.as_db() {
                Some(e) => { return Err(status::Custom(HTTPStatus::InternalServerError, format!("Failed to download bounds: {}", e))); },
                _ => { return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to download bounds"))); }
            }
        }
    };

    let mut rows = match stmt.lazy_query(&trans, &[&bounds], 1000) {
        Ok(rows) => rows,
        Err(err) => {
            match err.as_db() {
                Some(e) => { return Err(status::Custom(HTTPStatus::InternalServerError, format!("Failed to download bounds: {}", e))); },
                _ => { return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Failed to download bounds"))); }
            }
        }
    };

    //TODO: Due to the way rocket/postgres are written it makes it very difficult to wrap/pass an Iter
    //back to Rocket as a stream - instead use a file as an intermediary
    let dir = TempDir::new(&*&format!("bounds_get_{}", rand::random::<i32>().to_string())).unwrap();
    let file_path = dir.path().join("bounds.geojson");

    { 
        let mut f = File::create(&file_path).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let mut row: String = row.get(0);
            row.push_str("\n");
            f.write(&row.into_bytes()).unwrap();
        }
    }

    let f = File::open(file_path).unwrap();

    Ok(Stream::from(f))
}

#[get("/data/features?<map>")]
fn features_get(conn: DbConn, map: Map) -> Result<String, status::Custom<String>> {
    let bbox: Vec<f64> = map.bbox.split(',').map(|s| s.parse().unwrap()).collect();
    match feature::get_bbox(&conn.0, bbox) {
        Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[post("/data/features", format="application/json", data="<body>")]
fn features_action(auth: HTTPAuth, conn: DbConn, body: String) -> Result<Json, status::Custom<String>> {
    let uid = match user::auth(&conn.0, &auth.username, &auth.password) {
        Ok(Some(uid)) => uid,
        _ => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized!"))); }
    };

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
        match feature::action(&trans, &feat, &None) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Err(status::Custom(HTTPStatus::ExpectationFailed, err.to_string()));
            },
            Ok(res) => {
                if res.old == None {
                    feat.id = Some(json!(res.new));
                }
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
fn xml_map(conn: DbConn, map: Map) -> Result<String, status::Custom<String>> {
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
fn xml_changeset_create(auth: HTTPAuth, conn: DbConn, body: String) -> Result<String, status::Custom<String>> {
    let uid = match user::auth(&conn.0, &auth.username, &auth.password) {
        Ok(Some(uid)) => uid,
        _ => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized!"))); }
    };

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
fn xml_changeset_close(auth: HTTPAuth, conn: DbConn, id: i64) -> Result<String, status::Custom<String>> {
    match user::auth(&conn.0, &auth.username, &auth.password) {
        Ok(Some(_)) => Ok(id.to_string()),
        _ => Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized!")))
    }
}

#[put("/0.6/changeset/<delta_id>", data="<body>")]
fn xml_changeset_modify(auth: HTTPAuth, conn: DbConn, delta_id: i64, body: String) -> Result<status::Custom<String>, Response<'static>> {
    let uid = match user::auth(&conn.0, &auth.username, &auth.password) {
        Ok(Some(uid)) => uid,
        _ => { return Ok(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized!"))); }
    };

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
            return Err(conflict_response);
        }
    }

    let map = match xml::to_delta(&body) {
        Ok(map) => map,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Ok(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
        }
    };

    let delta_id = match delta::modify_props(&delta_id, &trans, &map, &uid) {
        Ok(id) => id,
        Err(err) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Ok(status::Custom(HTTPStatus::InternalServerError, err.to_string()));
        }
    };

    trans.commit().unwrap();

    Ok(status::Custom(HTTPStatus::Ok, delta_id.to_string()))
}

#[post("/0.6/changeset/<delta_id>/upload", data="<body>")]
fn xml_changeset_upload(auth: HTTPAuth, conn: DbConn, delta_id: i64, body: String) -> Result<status::Custom<String>, Response<'static>> {
    let uid = match user::auth(&conn.0, &auth.username, &auth.password) {
        Ok(Some(uid)) => uid,
        _ => { return Ok(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized!"))); }
    };

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
            return Err(conflict_response);
        }
    }

    let (mut fc, tree) = match xml::to_features(&body) {
        Ok(fctree) => fctree,
        Err(err) => { return Ok(status::Custom(HTTPStatus::ExpectationFailed, err.to_string())); }
    };

    let mut ids: HashMap<i64, feature::Response> = HashMap::new();

    for feat in &mut fc.features {
        let feat_res = match feature::action(&trans, &feat, &Some(delta_id)) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Ok(status::Custom(HTTPStatus::ExpectationFailed, err.to_string()));
            },
            Ok(feat_res) => {
                if feat_res.old == None {
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
            return Ok(status::Custom(HTTPStatus::InternalServerError, String::from("Could not format diffResult XML")));
        },
        Ok(diffres) => diffres
    };

    match delta::modify(&delta_id, &trans, &fc, &uid) {
        Ok (_) => (),
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Ok(status::Custom(HTTPStatus::InternalServerError, String::from("Could not create delta")));
        }
    }

    match delta::finalize(&delta_id, &trans) {
        Ok (_) => {
            trans.commit().unwrap();
            Ok(status::Custom(HTTPStatus::Ok, diffres))
        },
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            Ok(status::Custom(HTTPStatus::InternalServerError, String::from("Could not close delta")))
        }
    }
}

#[get("/capabilities")]
fn xml_capabilities() -> String {
    String::from("
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
    ")
}

#[get("/0.6/capabilities")]
fn xml_06capabilities() -> String {
   String::from("
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
    ")
}

#[get("/0.6/user/details")]
fn xml_user() -> String {
    String::from("
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
    ")
}

#[post("/data/feature", format="application/json", data="<body>")]
fn feature_action(auth: HTTPAuth, conn: DbConn, body: String) -> Result<Json, status::Custom<String>> {
    let uid = match user::auth(&conn.0, &auth.username, &auth.password) {
        Ok(Some(uid)) => uid,
        _ => { return Err(status::Custom(HTTPStatus::Unauthorized, String::from("Not Authorized!"))); }
    };

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

    match feature::action(&trans, &feat, &None) {
        Ok(res) => { feat.id = Some(json!(res.new)) },
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
fn feature_get(conn: DbConn, id: i64) -> Result<String, status::Custom<String>> {
    match feature::get(&conn.0, &id) {
        Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}
