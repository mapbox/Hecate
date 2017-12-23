extern crate hecate;
#[macro_use] extern crate clap;
#[macro_use] extern crate serde_json;
extern crate iron;
extern crate router;
extern crate geojson;
extern crate postgres;
extern crate mount;
extern crate persistent;
extern crate r2d2;
extern crate urlencoded;
extern crate r2d2_postgres;
extern crate logger;
extern crate env_logger;

use iron::typemap::Key;

//Postgres Connection Pooling
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

pub struct DB;
impl Key for DB { type Value = PostgresPool; }

use urlencoded::UrlEncodedQuery;
use clap::App;
use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::Read;
use std::collections::HashMap;
use geojson::GeoJson;
use hecate::feature;
use hecate::delta;
use hecate::xml;
use mount::Mount;
use logger::Logger;

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let _matched = App::from_yaml(cli_cnf).get_matches();

    env_logger::init().unwrap();

    //Create Postgres Connection Pool
    let cn_str = String::from("postgres://postgres@localhost:5432/hecate");
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(cn_str, TlsMode::None).unwrap();
    let config = ::r2d2::Config::builder().pool_size(6).build();
    let pool = ::r2d2::Pool::new(config, manager).unwrap();

    let (logger_before, logger_after) = Logger::new(None);

    let mut router = Router::new();

    router.get("/", index, "index");

    //router.post("/api/user/create", user_create), "user_create");
    //router.get("/api/user/token", user_token), "user_token");

    // Create Modify Delete Individual Features
    // Each must have a valid GeoJSON Feature in the body
    router.post("/api/data/feature", feature_create, "feature_create");
    router.patch("/api/data/feature", feature_modify, "feature_modify");
    router.delete("/api/data/feature", feature_delete, "feature_delete");

    // Create Edit Modify Batch Features
    // Must have a valid GeoJSON FeatureCollection in the body
    router.post("/api/data/features", features_action, "features_action");

    // Get Features
    router.get("/api/data/feature/:feature", feature_get, "feature_get"); //Get by id
    router.get("/api/data/features", features_get, "features_get"); //Get by bounding box

    //OSM XML Compat. Shim & Obviously incomplete as the OSM data model can't translate 1:1
    router.get("/api/capabilities", xml_capabilities, "xml_capabilities");
    router.get("/api/0.6/capabilities", xml_capabilities, "xml_06capabilities");
    router.get("/api/0.6/user/details", xml_user, "xml_06user");
    router.get("http://localhost:3000/api/0.6", index, "xml");
    router.get("/api/0.6/map", xml_map, "xml_map");
    router.put("/api/0.6/changeset/create", xml_changeset_create, "xml_createChangeset");
    router.put("/api/0.6/changeset/:id", xml_changeset_modify, "xml_modifyChangeset");
    router.post("/api/0.6/changeset/:id/upload", xml_changeset_upload, "xml_putChangeset");
    router.put("/api/0.6/changeset/:id/close", xml_changeset_close, "xml_closeChangeset");

    let mut mount = Mount::new();
    mount.mount("/", router);

    let mut chain = Chain::new(mount);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link(persistent::Read::<DB>::both(pool));
    Iron::new(chain).http("localhost:3000").unwrap();
}

fn index(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok)))
}

fn features_get(req: &mut Request) -> IronResult<Response> {
    let bbox_error = Response::with((status::BadRequest, "single bbox query param required"));

    let query = match req.get_ref::<UrlEncodedQuery>() {
        Ok(hashmap) => {
            match hashmap.get("bbox") {
                Some(bbox) => {
                    if bbox.len() != 1 { return Ok(bbox_error); }

                    let split: Vec<f64> = bbox[0].split(',').map(|s| s.parse().unwrap()).collect();

                    split
                },
                None => { return Ok(bbox_error); }
            }
        },
        Err(_) => { return Ok(bbox_error); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::get_bbox(&conn, query) {
        Ok(features) => Ok(Response::with((status::Ok, geojson::GeoJson::from(features).to_string()))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn features_action(req: &mut Request) -> IronResult<Response> {
    let mut fc = match get_geojson(req) {
        Ok(GeoJson::FeatureCollection(fc)) => fc,
        Ok(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON FeatureCollection"))); }
        Err(err) => { return Ok(err); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let map: HashMap<String, Option<String>> = HashMap::new();
    let delta_id = match delta::open(&trans, &map, &1) {
        Ok(id) => id,
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not create delta"))); }
    };

    for feat in &mut fc.features {
        match feature::action(&trans, &feat, &None) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Ok(Response::with((status::ExpectationFailed, err.to_string())));
            },
            Ok(res) => {
                if res.old == None {
                    feat.id = Some(json!(res.new));
                }
            }
        }
    }

    if delta::modify(&delta_id, &trans, &fc, &1).is_err() {
        return Ok(Response::with((status::InternalServerError, "Could not create delta")));
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    }
}

fn xml_map(req: &mut Request) -> IronResult<Response> {
    let bbox_error = Response::with((status::BadRequest, "single bbox query param required"));

    let query = match req.get_ref::<UrlEncodedQuery>() {
        Ok(hashmap) => {
            match hashmap.get("bbox") {
                Some(bbox) => {
                    if bbox.len() != 1 { return Ok(bbox_error); }

                    let split: Vec<f64> = bbox[0].split(',').map(|s| s.parse().unwrap()).collect();

                    split
                },
                None => { return Ok(bbox_error); }
            }
        },
        Err(_) => { return Ok(bbox_error); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    let fc = match feature::get_bbox(&conn, query) {
        Ok(features) => features,
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))) }
    };

    let xml_str = match xml::from_features(&fc) {
        Ok(xml_str) => xml_str,
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))) }
    };

    Ok(Response::with((status::Ok, xml_str)))
}

fn xml_changeset_create(req: &mut Request) -> IronResult<Response> {
    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let map = match xml::to_delta(&body_str) {
        Ok(map) => map,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let delta_id = match delta::open(&trans, &map, &1) {
        Ok(id) => id,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    trans.commit().unwrap();

    Ok(Response::with((status::Ok, delta_id.to_string())))
}

fn xml_changeset_close(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, String::from("true"))))
}

fn xml_changeset_modify(req: &mut Request) -> IronResult<Response> {
    let delta_id: i64 = match req.extensions.get::<Router>().unwrap().find("id") {
        Some(id) => match id.parse() {
            Ok(id) => id,
            Err(_) =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be numeric"))); }
        },
        None =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be provided"))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let mut conflict_resp = Response::with((status::Conflict, format!("The changeset {} was closed at previously", &delta_id)));
    conflict_resp.headers.append_raw("Error", (&*format!("The changeset {} was closed at previously", &delta_id)).as_bytes().to_vec());

    match delta::is_open(&delta_id, &trans) {
        Err(_err) => { return Ok(conflict_resp); },
        Ok(false) => { return Ok(conflict_resp); },
        Ok(true) => ()
    }

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let map = match xml::to_delta(&body_str) {
        Ok(map) => map,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    let delta_id = match delta::modify_props(&delta_id, &trans, &map, &1) {
        Ok(id) => id,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    trans.commit().unwrap();

    Ok(Response::with((status::Ok, delta_id.to_string())))
}

//TODO
// - INSERT FEAT INTO CHANGESET
// - CHECK THAT CHANGESET EXISTS
// - CHECK THAT CHANGESET IS NOT FINALIZED
fn xml_changeset_upload(req: &mut Request) -> IronResult<Response> {
    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let delta_id: i64 = match req.extensions.get::<Router>().unwrap().find("id") {
        Some(id) => match id.parse() {
            Ok(id) => id,
            Err(_) =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be numeric"))); }
        },
        None =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be provided"))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let mut conflict_resp = Response::with((status::Conflict, format!("The changeset {} was closed at previously", &delta_id)));
    conflict_resp.headers.append_raw("Error", (&*format!("The changeset {} was closed at previously", &delta_id)).as_bytes().to_vec());

    match delta::is_open(&delta_id, &trans) {
        Err(_) => { return Ok(conflict_resp); },
        Ok(false) => { return Ok(conflict_resp); },
        Ok(true) => ()
    }

    let (mut fc, tree) = match xml::to_features(&body_str) {
        Ok(fctree) => fctree,
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))); }
    };

    let mut ids: HashMap<i64, feature::Response> = HashMap::new();

    for feat in &mut fc.features {
        let feat_res = match feature::action(&trans, &feat, &Some(delta_id)) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Ok(Response::with((status::ExpectationFailed, err.to_string())));
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
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not format diffResult XML"))); },
        Ok(diffres) => diffres
    };

    match delta::modify(&delta_id, &trans, &fc, &1) {
        Ok (_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, diffres)))
        },
        Err(_) => Ok(Response::with((status::InternalServerError, "Could not create delta")))
    }
}

fn xml_capabilities(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "
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
    ")))
}

fn xml_user(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "
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
    ")))
}

fn feature_create(req: &mut Request) -> IronResult<Response> {
    let mut feat = match get_geojson(req) {
        Ok(GeoJson::Feature(feat)) => feat,
        Ok(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
        Err(err) => { return Ok(err); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let map: HashMap<String, Option<String>> = HashMap::new();
    let delta_id = match delta::open(&trans, &map, &1) {
        Ok(id) => id,
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not create delta"))); }
    };

    match feature::create(&trans, &feat, &None) {
        Ok(res) => { feat.id = Some(json!(res.new)) },
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))); }
    }

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat ],
        foreign_members: None,
    };

    if delta::modify(&delta_id, &trans, &fc, &1).is_err() {
        return Ok(Response::with((status::InternalServerError, "Could not create delta")));
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    }
}

fn feature_modify(req: &mut Request) -> IronResult<Response> {
    let feat = match get_geojson(req) {
        Ok(GeoJson::Feature(feat)) => feat,
        Ok(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
        Err(err) => { return Ok(err); }
    };

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat.clone() ],
        foreign_members: None,
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let map: HashMap<String, Option<String>> = HashMap::new();

    let delta_id = match delta::create(&trans, &fc, &map, &1) {
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not create delta"))); },
        Ok(id) => id
    };

    match feature::modify(&trans, &feat, &None) {
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))); },
        _ => ()
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    }
}

fn feature_get(req: &mut Request) -> IronResult<Response> {
    let feature_id: i64 = match req.extensions.get::<Router>().unwrap().find("feature") {
        Some(id) => match id.parse() {
            Ok(id) => id,
            Err(_) =>  { return Ok(Response::with((status::ExpectationFailed, "Feature ID Must be numeric"))); }
        },
        None =>  { return Ok(Response::with((status::ExpectationFailed, "Feature ID Must be provided"))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::get(&conn, &feature_id) {
        Ok(features) => Ok(Response::with((status::Ok, geojson::GeoJson::from(features).to_string()))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn feature_delete(req: &mut Request) -> IronResult<Response> {
    let feat = match get_geojson(req) {
        Ok(GeoJson::Feature(feat)) => feat,
        Ok(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
        Err(err) => { return Ok(err); }
    };

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat.clone() ],
        foreign_members: None,
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let map: HashMap<String, Option<String>> = HashMap::new();

    let delta_id = match delta::create(&trans, &fc, &map, &1) {
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not create delta"))); },
        Ok(id) => id
    };

    match feature::delete(&trans, &feat) {
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))); },
        _ => ()
    }

    match delta::finalize(&delta_id, &trans) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    }
}

fn get_geojson(req: &mut Request) -> Result<geojson::GeoJson, Response> {
    match req.headers.get::<iron::headers::ContentType>() {
        Some(ref header) => {
            if **header != iron::headers::ContentType::json() {
                return Err(Response::with((status::UnsupportedMediaType, "ContentType must be application/json")));
            }
        },
        None => { return Err(Response::with((status::UnsupportedMediaType, "ContentType must be application/json"))); }
    }

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let geojson = match body_str.parse::<GeoJson>() {
        Err(_) => { return Err(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); },
        Ok(geo) => geo
    };

    Ok(geojson)
}
