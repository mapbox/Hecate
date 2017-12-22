#[macro_use] extern crate clap;
#[macro_use] extern crate serde_json;
extern crate iron;
extern crate router;
extern crate geojson;
extern crate hecate;
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
use hecate::changeset;
use hecate::xml;
use mount::Mount;
use logger::Logger;

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let _matched = App::from_yaml(cli_cnf).get_matches();

    env_logger::init().unwrap();

    let cn_str = String::from("postgres://postgres@localhost:5432/hecate");
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(cn_str, TlsMode::None).unwrap();
    let config = ::r2d2::Config::builder().pool_size(6).build();
    let pool = ::r2d2::Pool::new(config, manager).unwrap();

    let (logger_before, logger_after) = Logger::new(None);

    let mut router = Router::new();

    router.get("/", index, "index");

    // Create Edit Modify Feature
    router.post("/api/data/feature", feature_create, "feature_create");
    router.patch("/api/data/feature", feature_modify, "feature_modify");
    router.delete("/api/data/feature", feature_delete, "feature_delete");

    // Create Edit Modify Feature
    router.post("/api/data/features", features_action, "features_action");

    // Get Features
    router.get("/api/data/feature/:feature", feature_get, "feature_get");
    router.get("/api/data/features", features_get, "features_get");

    //OSM XML Compat. Shim
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

    let changeset_id = match changeset::open(&trans, &map, &1) {
        Ok(id) => id,
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not create changeset"))); }
    };

    for feat in &mut fc.features {
        match feature::action(&trans, &feat, &None) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Ok(Response::with((status::ExpectationFailed, err.to_string())));
            },
            Ok(res) => {
                //If res.old is 0 then the feature is being created - assign it the id
                //so that the changeset can enter it into the affected array
                if res.old == 0 {
                    feat.id = Some(json!(res.new));
                }
            }
        }
    }

    match changeset::modify(&changeset_id, &trans, &fc, &map, &1) {
        Ok (_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(_) => Ok(Response::with((status::InternalServerError, "Could not create changeset")))
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

    let map = match xml::to_changeset(&body_str) {
        Ok(map) => map,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ ],
        foreign_members: None,
    };


    let id = match changeset::create(&trans, &fc, &map, &1) {
        Ok(id) => id,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    trans.commit().unwrap();

    Ok(Response::with((status::Ok, id.to_string())))
}

fn xml_changeset_close(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, String::from("true"))))
}

fn xml_changeset_modify(req: &mut Request) -> IronResult<Response> {
    let changeset_id: i64 = match req.extensions.get::<Router>().unwrap().find("id") {
        Some(id) => match id.parse() {
            Ok(id) => id,
            Err(_) =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be numeric"))); }
        },
        None =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be provided"))); }
    };

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let map = match xml::to_changeset(&body_str) {
        Ok(map) => map,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ ],
        foreign_members: None,
    };

    let id = match changeset::modify(&changeset_id, &trans, &fc, &map, &1) {
        Ok(id) => id,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    trans.commit().unwrap();

    Ok(Response::with((status::Ok, id.to_string())))
}

fn  xml_changeset_upload(req: &mut Request) -> IronResult<Response> {
    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let changeset_id: i64 = match req.extensions.get::<Router>().unwrap().find("id") {
        Some(id) => match id.parse() {
            Ok(id) => id,
            Err(_) =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be numeric"))); }
        },
        None =>  { return Ok(Response::with((status::ExpectationFailed, "Changeset ID Must be provided"))); }
    };

    let (fc, tree) = match xml::to_features(&body_str) {
        Ok(fctree) => fctree,
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let mut ids: HashMap<i64, feature::Response> = HashMap::new();

    for feat in fc.features {
        let feat_res = match feature::action(&trans, &feat, &Some(changeset_id)) {
            Err(err) => {
                trans.set_rollback();
                trans.finish().unwrap();
                return Ok(Response::with((status::ExpectationFailed, err.to_string())));
            },
            Ok(feat_res) => feat_res
        };

        ids.insert(feat_res.old, feat_res);
    }

    let diffres = match xml::to_diffresult(ids, tree) {
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not format diffResult XML"))); },
        Ok(diffres) => diffres
    };

    trans.commit().unwrap();

    Ok(Response::with((status::Ok, diffres)))
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

    let changeset_id = match changeset::open(&trans, &map, &1) {
        Ok(id) => id,
        Err(_) => { return Ok(Response::with((status::InternalServerError, "Could not create changeset"))); }
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

    match changeset::modify(&changeset_id, &trans, &fc, &map, &1) {
        Ok (_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(_) => Ok(Response::with((status::InternalServerError, "Could not create changeset")))
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
    if changeset::create(&trans, &fc, &map, &1).is_err() {
        return Ok(Response::with((status::InternalServerError, "Could not create changeset")));
    }

    match feature::modify(&trans, &feat, &None) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
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
    if changeset::create(&trans, &fc, &map, &1).is_err() {
        return Ok(Response::with((status::InternalServerError, "Could not create changeset")));
    }

    match feature::delete(&trans, &feat) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
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
