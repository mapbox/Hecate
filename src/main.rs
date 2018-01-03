#![feature(plugin)]
#![feature(custom_derive)]
#![feature(custom_attribute)]
#![feature(attr_literals)]
#![plugin(rocket_codegen)]

extern crate hecate;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate clap;
#[macro_use] extern crate serde_json;
extern crate geojson;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate env_logger;

//Postgres Connection Pooling
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use rocket_contrib::Json as Json;
use rocket::response::status;
use std::io::Cursor;
use rocket::http::Status as HTTPStatus;
use rocket::{Request, State, Outcome};
use rocket::response::Response;
use rocket::request::{self, FromRequest};
use clap::App;
use std::collections::HashMap;
use geojson::GeoJson;
use hecate::feature;
use hecate::user;
use hecate::delta;
use hecate::xml;

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

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let matched = App::from_yaml(cli_cnf).get_matches();

    let database = matched.value_of("database").unwrap_or("postgres@localhost:5432/hecate");

    env_logger::init().unwrap();

    rocket::ignite()
        .manage(init_pool(&database))
        .mount("/", routes![index])
        .mount("/api", routes![
            user_create,
            feature_action,
            features_action,
            feature_get,
            features_get,
            xml_capabilities,
            xml_06capabilities,
            xml_user,
            xml_map,
            xml_changeset_create,
            xml_changeset_modify,
            xml_changeset_upload,
            xml_changeset_close
        ])
        .catch(errors![not_found])
        .launch();
}

#[get("/")]
fn index() -> &'static str {
    "Hello World!"
}

#[error(404)]
fn not_found() -> Json {
    Json(json!({
        "status": "error",
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

#[get("/user/create?<user>")]
fn user_create(conn: DbConn, user: User) -> Result<Json, status::Custom<String>> {
    user::create(&conn.0, &user.username, &user.password, &user.email);
    Ok(Json(json!(true)))
}

#[get("/data/features?<map>")]
fn features_get(conn: DbConn, map: Map) -> Result<String, status::Custom<String>> {
    let bbox: Vec<f64> = map.bbox.split(',').map(|s| s.parse().unwrap()).collect();
    match feature::get_bbox(&conn.0, bbox) {
        Ok(features) => Ok(geojson::GeoJson::from(features).to_string()),
        Err(err) => Err(status::Custom(HTTPStatus::BadRequest, err.to_string()))
    }
}

#[post("/data/features", data="<body>")]
fn features_action(conn: DbConn, body: String) -> Result<Json, status::Custom<String>> {
    let mut fc = match body.parse::<GeoJson>() {
        Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON Feature"))); },
        Ok(geo) => match geo {
            GeoJson::FeatureCollection(fc) => fc,
            _ => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON FeatureCollection"))); }
        }
    };

    let trans = conn.0.transaction().unwrap();

    let map: HashMap<String, Option<String>> = HashMap::new();
    let delta_id = match delta::open(&trans, &map, &1) {
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

    if delta::modify(&delta_id, &trans, &fc, &1).is_err() {
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
fn xml_changeset_create(conn: DbConn, body: String) -> Result<String, status::Custom<String>> {
    let map = match xml::to_delta(&body) {
        Ok(map) => map,
        Err(err) => { return Err(status::Custom(HTTPStatus::InternalServerError, err.to_string())); }
    };

    let trans = conn.0.transaction().unwrap();

    let delta_id = match delta::open(&trans, &map, &1) {
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
fn xml_changeset_close(id: i64) -> String {
    id.to_string()
}

#[put("/0.6/changeset/<delta_id>", data="<body>")]
fn xml_changeset_modify(conn: DbConn, delta_id: i64, body: String) -> Result<status::Custom<String>, Response<'static>> {
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

    let delta_id = match delta::modify_props(&delta_id, &trans, &map, &1) {
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
fn xml_changeset_upload(conn: DbConn, delta_id: i64, body: String) -> Result<status::Custom<String>, Response<'static>> {
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

    match delta::modify(&delta_id, &trans, &fc, &1) {
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
fn feature_action(conn: DbConn, body: String) -> Result<Json, status::Custom<String>> {
    let mut feat = match body.parse::<GeoJson>() {
        Err(_) => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON Feature"))); },
        Ok(geo) => match geo {
            GeoJson::Feature(feat) => feat,
            _ => { return Err(status::Custom(HTTPStatus::BadRequest, String::from("Body must be valid GeoJSON Feature"))); }
        }
    };

    let trans = conn.0.transaction().unwrap();

    let map: HashMap<String, Option<String>> = HashMap::new();
    let delta_id = match delta::open(&trans, &map, &1) {
        Ok(id) => id,
        Err(_) => {
            trans.set_rollback();
            trans.finish().unwrap();
            return Err(status::Custom(HTTPStatus::InternalServerError, String::from("Could not create delta")));
        }
    };

    match feature::create(&trans, &feat, &None) {
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

    if delta::modify(&delta_id, &trans, &fc, &1).is_err() {
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
