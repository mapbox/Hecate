extern crate geojson;
extern crate quick_xml;
extern crate serde_json;

mod node;
mod way;
mod rel;
mod tree;

use feature;
use xml::node::Node;
use xml::way::Way;
use xml::rel::Rel;
use xml::tree::OSMTree;

use std::string;
use std::num;
use std::io::Cursor;
use self::quick_xml::writer::Writer;
use self::quick_xml::events as XMLEvents;
use std::collections::HashMap;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum XMLError {
    Unknown,
    Invalid,
    GCNotSupported,
    EncodingFailed,
    InternalError(String),
    ParsingError,
    InvalidNode(String),
    InvalidNodeRef,
    InvalidWay(String),
    InvalidWayRef,
    InvalidRel(String),
    InvalidXML,
    InvalidFeature,
    NotFoundError,
    StringParsing(string::ParseError),
    IntParsing(num::ParseIntError),
    FloatParsing(num::ParseFloatError)
}

impl XMLError {
    pub fn to_string(&self) -> String {
        match *self {
            XMLError::Unknown => String::from("Unknown Error"),
            XMLError::GCNotSupported => String::from("GeometryCollection are not currently supported"),
            XMLError::Invalid => String::from("Could not parse XML - Invalid"),
            XMLError::EncodingFailed => String::from("Encoding Failed"),
            XMLError::InternalError(ref msg) => format!("Internal Error: {}", msg),
            XMLError::ParsingError => String::from("Parsing Error"),
            XMLError::InvalidNode(ref msg) => format!("Invalid Node: {}", msg),
            XMLError::InvalidNodeRef => String::from("Invalid Node Reference"),
            XMLError::InvalidWay(ref msg) => format!("Invalid Way: {}", msg),
            XMLError::InvalidWayRef => String::from("Invalid Way Reference"),
            XMLError::InvalidRel(ref msg) => format!("Invalid Relation: {}", msg),
            XMLError::InvalidXML => String::from("Invalid XML"),
            XMLError::NotFoundError => String::from("Not Found"),
            XMLError::InvalidFeature => String::from("Invalid Feature"),
            XMLError::StringParsing(_) => String::from("Could not parse attribute to string"),
            XMLError::IntParsing(_) => String::from("Could not parse attribute to integer"),
            XMLError::FloatParsing(_) => String::from("Could not parse attribute to float")
        }
    }
}

impl From<string::FromUtf8Error> for XMLError {
    fn from(error: string::FromUtf8Error) -> XMLError {
        XMLError::ParsingError
    }
}

impl From<quick_xml::errors::Error> for XMLError {
    fn from(error: quick_xml::errors::Error) -> XMLError {
        XMLError::InvalidXML
    }
}

impl From<string::ParseError> for XMLError {
    fn from(error: string::ParseError) -> XMLError {
        XMLError::StringParsing(error)
    }
}

impl From<num::ParseFloatError> for XMLError {
    fn from(error: num::ParseFloatError) -> XMLError {
        XMLError::FloatParsing(error)
    }
}

impl From<num::ParseIntError> for XMLError {
    fn from(error: num::ParseIntError) -> XMLError {
        XMLError::IntParsing(error)
    }
}

#[derive(PartialEq, Debug)]
pub enum Value {
    None,
    Node,
    Way,
    Rel
}

#[derive(PartialEq, Debug, Clone)]
pub enum Action {
    None,
    Create,
    Modify,
    Delete
}

pub trait Generic {
    fn new() -> Self;
    fn value(&self) -> Value;
    fn set_tag(&mut self, k: String, v: String);
    fn has_tags(&self) -> bool;
    fn to_feat(&self, tree: &OSMTree) -> Result<geojson::Feature, XMLError>;
    fn is_valid(&self) -> Result<bool, String>;
}

pub struct OSMTypes {
    node_it: i64,
    nodes: String,
    ways: String,
    rels: String
}

impl OSMTypes {
    pub fn new() -> OSMTypes {
        OSMTypes {
            node_it: 8000000000000000000,
            nodes: String::from(""),
            ways: String::from(""),
            rels: String::from("")
        }
    }
}

pub fn to_diffresult(ids: HashMap<i64, feature::Response>, tree: OSMTree) -> Result<String, XMLError> {
    let mut diffres = String::from(r#"<diffResult generator="Hecate Server" version="0.6">"#);

    for (_i, n) in tree.get_nodes() {
        if n.action == Some(Action::Create) {
            match ids.get(&n.id.unwrap()) {
                Some(diffid) => { diffres.push_str(&*format!(r#"<node old_id="{}" new_id="{}" new_version="{}"/>"#, diffid.old.unwrap(), diffid.new.unwrap(), diffid.version.unwrap())); },
                _ => ()
            }
        } else if n.action == Some(Action::Modify) {
            match ids.get(&n.id.unwrap()) {
                Some(diffid) => { diffres.push_str(&*format!(r#"<node old_id="{}" new_id="{}" new_version="{}"/>"#, n.id.unwrap(), n.id.unwrap(), diffid.version.unwrap())); },
                _ => ()
            }
        } else if n.action == Some(Action::Delete) {
            match ids.get(&n.id.unwrap()) {
                Some(_) => { diffres.push_str(&*format!(r#"<node old_id="{}"/>"#, n.id.unwrap())); },
                _ => ()
            }
        }
    }

    for (_i, w) in tree.get_ways() {
        if w.action == Some(Action::Create) {
            match ids.get(&w.id.unwrap()) {
                Some(diffid) => { diffres.push_str(&*format!(r#"<way old_id="{}" new_id="{}" new_version="{}"/>"#, diffid.old.unwrap(), diffid.new.unwrap(), diffid.version.unwrap())); },
                _ => ()
            }
        } else if w.action == Some(Action::Modify) {
            match ids.get(&w.id.unwrap()) {
                Some(diffid) => { diffres.push_str(&*format!(r#"<way old_id="{}" new_id="{}" new_version="{}"/>"#, w.id.unwrap(), w.id.unwrap(), diffid.version.unwrap())); },
                _ => ()
            }
        } else if w.action == Some(Action::Delete) {
            match ids.get(&w.id.unwrap()) {
                Some(_) => { diffres.push_str(&*format!(r#"<way old_id="{}"/>"#, w.id.unwrap())); },
                _ => ()
            }
        }

    }

    for (_i, _r) in tree.get_rels() {

    }

    diffres.push_str(r#"</diffResult>"#);

    Ok(diffres)
}

pub fn to_delta_tag(xml_node: &quick_xml::events::BytesStart, map: &mut HashMap<String, Option<String>>) { let mut kv: (Option<String>, Option<String>) = (None, None);
    for attr in xml_node.attributes() {
        let attr = attr.unwrap();

        match attr.key {
            b"k"  => kv.0 = Some(String::from_utf8_lossy(attr.value).parse().unwrap()),
            b"v"  => kv.1 = Some(String::from_utf8_lossy(attr.value).parse().unwrap()),
            _ => ()
        }
    }

    map.insert(kv.0.unwrap(), kv.1);
}

pub fn to_delta(body: &String) -> Result<HashMap<String, Option<String>>, XMLError> {
    let mut reader = quick_xml::reader::Reader::from_str(body);
    let mut buf = Vec::new();

    let mut map = HashMap::new();

     loop {
        match reader.read_event(&mut buf) {
            Ok(XMLEvents::Event::Start(ref e)) => {
                match e.name() {
                    b"tag" => { to_delta_tag(&e, &mut map) },
                    _ => (),
                }
            },
            Ok(XMLEvents::Event::Empty(ref e)) => {
                match e.name() {
                    b"tag" => { to_delta_tag(&e, &mut map) },
                    _ => (),
                }
            },
            Ok(XMLEvents::Event::Eof) => { break; },
            Err(_) => { return Err(XMLError::Invalid); },
            _ => ()
        }

        buf.clear()
    }

    Ok(map)
}

pub fn to_features(body: &String) -> Result<(geojson::FeatureCollection, OSMTree), XMLError> {
    let tree = tree_parser(&body)?;

    let mut fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![],
        foreign_members: None
    };

    for (_i, rel) in tree.get_rels() {
        if rel.action != Some(Action::Delete) && !rel.has_tags() { continue; }

        fc.features.push(rel.to_feat(&tree)?);
    }

    for (_i, way) in tree.get_ways() {
        if way.action != Some(Action::Delete) && !way.has_tags() { continue; }

        fc.features.push(way.to_feat(&tree)?);
    }

    for (_i, node) in tree.get_nodes() {
        if node.action != Some(Action::Delete) && !node.has_tags() { continue; }

        let n = node.to_feat(&tree)?;

        fc.features.push(n);
    }

    Ok((fc, tree))
}

pub fn tree_parser(body: &String) -> Result<OSMTree, XMLError> {
    let mut tree: OSMTree = OSMTree::new();

    let mut opening_osm = false;
    let mut n: Node = Node::new();
    let mut w: Way = Way::new();
    let mut r: Rel = Rel::new();

    let mut current_value = Value::None;
    let mut current_action = Action::None;

    let mut reader = quick_xml::reader::Reader::from_str(body);
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(XMLEvents::Event::Start(ref e)) => {
                match e.name() {
                    b"osmChange" => {
                        parse_osm(&e, &mut tree.meta)?;
                        opening_osm = true;
                    },
                    b"create" => {
                        if current_action != Action::None { return Err(XMLError::InternalError(String::from("Action Already Specialized"))); }
                        current_action = Action::Create;
                    },
                    b"modify" => {
                        if current_action != Action::None { return Err(XMLError::InternalError(String::from("Action Already Specialized"))); }
                        current_action = Action::Modify;
                    },
                    b"delete" => {
                        if current_action != Action::None { return Err(XMLError::InternalError(String::from("Action Already Specialized"))); }
                        current_action = Action::Delete;
                    },
                    b"node" => {
                        if current_action == Action::None { return Err(XMLError::InternalError(String::from("node must be in Action"))); }
                        if current_value != Value::None { return Err(XMLError::InternalError(String::from("node cannot be within another value"))); }

                        n = parse_node(e)?;
                        n.action = Some(current_action.clone());

                        if n.action == Some(Action::Create) {
                            n.version = Some(1);
                        }

                        current_value = Value::Node;
                    },
                    b"way" => {
                        if current_action == Action::None { return Err(XMLError::InternalError(String::from("way must be in Action"))); }
                        if current_value != Value::None { return Err(XMLError::InternalError(String::from("way cannot be within another value"))); }

                        w = parse_way(e)?;
                        w.action = Some(current_action.clone());

                        if w.action == Some(Action::Create) {
                            w.version = Some(1);
                        }

                        current_value = Value::Way;
                    },
                    b"relation" => {
                        if current_action == Action::None { return Err(XMLError::InternalError(String::from("rel must be in Action"))); }
                        if current_value != Value::None { return Err(XMLError::InternalError(String::from("rel cannot be within another value"))); }

                        r = parse_rel(e)?;
                        r.action = Some(current_action.clone());

                        if r.action == Some(Action::Create) {
                            r.version = Some(1);
                        }

                        current_value = Value::Rel;
                    },
                    b"tag" => {
                        let (k, v) = parse_tag(&e)?;

                        match current_value {
                            Value::None => { return Err(XMLError::InternalError(String::from("tags must be in value"))); },
                            Value::Node => {
                                n.set_tag(k, v);
                            },
                            Value::Way => {
                                w.set_tag(k, v);
                            },
                            Value::Rel => {
                                r.set_tag(k, v);
                            }
                        };

                    }
                    _ => (),
                }
            },
            Ok(XMLEvents::Event::Empty(ref e)) => {
                match e.name() {
                    b"node" => {
                        if current_action == Action::None { return Err(XMLError::InternalError(String::from("node must be in Action"))); }
                        if current_value != Value::None { return Err(XMLError::InternalError(String::from("node cannot be within another value"))); }

                        n = parse_node(&e)?;
                        n.action = Some(current_action.clone());

                        if n.action == Some(Action::Create) {
                            n.version = Some(1);
                        }

                        tree.add_node(n)?;

                        n = Node::new();
                    },
                    b"way" => {
                        return Err(XMLError::InternalError(String::from("ways cannot be self closing")));
                    },
                    b"relation" => {
                        return Err(XMLError::InternalError(String::from("rels cannot be self closing")));
                    },
                    b"nd" => {
                        if current_value != Value::Way { return Err(XMLError::InternalError(String::from("nd must be in way"))); }

                        let ndref = parse_nd(&e)?;
                        w.nodes.push(ndref);
                    },
                    b"tag" => {
                        let (k, v) = parse_tag(&e)?;

                        match current_value {
                            Value::None => { return Err(XMLError::InternalError(String::from("tags must be in value"))); },
                            Value::Node => {
                                n.set_tag(k, v);
                            },
                            Value::Way => {
                                w.set_tag(k, v);
                            },
                            Value::Rel => {
                                r.set_tag(k, v);
                            }
                        };
                    },
                    b"member" => {
                        let (rtype, rref, rrole) = parse_member(&e)?;

                        match current_value {
                            Value::Rel => {
                                r.set_member(rtype, rref, rrole);
                            },
                            _ => { return Err(XMLError::InternalError(String::from("member must be in rel"))); }
                        };

                    }
                    _ => ()
                }
            },
            Ok(XMLEvents::Event::End(ref e)) => {
                match e.name() {
                    b"node" => {
                        if current_value != Value::Node { return Err(XMLError::InternalError(String::from("node close outside of node"))); }

                        tree.add_node(n)?;
                        n = Node::new();
                        current_value = Value::None;
                    },
                    b"way" => {
                        if current_value != Value::Way { return Err(XMLError::InternalError(String::from("way close outside of node"))); }
                        tree.add_way(w)?;
                        w = Way::new();
                        current_value = Value::None;
                    },
                    b"relation" => {
                        if current_value != Value::Rel { return Err(XMLError::InternalError(String::from("rel close outside of node"))); }
                        tree.add_rel(r)?;
                        r = Rel::new();
                        current_value = Value::None;
                    },
                    b"create" => {
                        if current_action != Action::Create { return Err(XMLError::InternalError(String::from("create close outside of create"))); }
                        current_action = Action::None;
                    },
                    b"modify" => {
                        if current_action != Action::Modify { return Err(XMLError::InternalError(String::from("modify close outside of create"))); }
                        current_action = Action::None;
                    },
                    b"delete" => {
                        if current_action != Action::Delete { return Err(XMLError::InternalError(String::from("delete close outside of create"))); }
                        current_action = Action::None;
                    },
                    b"osmChange" => {
                        if current_value != Value::None { return Err(XMLError::InternalError(String::from("All values must be finished before osm close"))); }
                        if !opening_osm { return Err(XMLError::InternalError(String::from("osm close outside of osm"))); }

                        return Ok(tree);
                    }
                    _ => ()
                }
            }
            Ok(XMLEvents::Event::Eof) => { return Err(XMLError::Invalid); },
            Err(_) => { return Err(XMLError::Invalid); },
            _ => ()
        }

        buf.clear();
    }
}

pub fn from_features(fc: &geojson::FeatureCollection) -> Result<String, XMLError> {
    let mut xml: String = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><osm version="0.6" generator="ROSM">"#);
    let mut osm = OSMTypes::new();

    for feat in &fc.features {
        match feat.geometry {
            Some(ref geom) => {
                match geom.value {
                    geojson::Value::Point(ref coords) => {
                        point(&feat, &coords, &mut osm)?;
                    },
                    geojson::Value::MultiPoint(ref coords) => {
                        multipoint(&feat, &coords, &mut osm);
                    },
                    geojson::Value::LineString(ref coords) => {
                        linestring(&feat, &coords, &mut osm)?;
                    },
                    geojson::Value::MultiLineString(ref coords) => {
                        multilinestring(&feat, &coords, &mut osm);
                    },
                    geojson::Value::Polygon(ref coords) => {
                        polygon(&feat, &coords, &mut osm);
                    },
                    geojson::Value::MultiPolygon(ref coords) => {
                        multipolygon(&feat, &coords, &mut osm);
                    },
                    _ => { return Err(XMLError::GCNotSupported); }
                }
            },
            None => { return Err(XMLError::Unknown); }
        }
    }

    xml.push_str(&*osm.nodes);
    xml.push_str(&*osm.ways);
    xml.push_str(&*osm.rels);
    xml.push_str("</osm>");

    Ok(xml)
}

pub fn point(feat: &geojson::Feature, coords: &geojson::PointType, osm: &mut OSMTypes) -> Result<bool, XMLError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut xml_node = XMLEvents::BytesStart::owned(b"node".to_vec(), 4);

    xml_node.push_attribute(("id", &*feature::get_id(feat).unwrap().to_string()));
    xml_node.push_attribute(("version", &*feature::get_version(feat).unwrap().to_string()));
    xml_node.push_attribute(("lon", &*coords[0].to_string()));
    xml_node.push_attribute(("lat", &*coords[1].to_string()));

    writer.write_event(XMLEvents::Event::Start(xml_node)).unwrap();

    match *&feat.properties {
        Some(ref props) => {
            for (k, v) in props.iter() {
                let mut xml_tag = XMLEvents::BytesStart::owned(b"tag".to_vec(), 3);
                xml_tag.push_attribute(("k", k.as_str()));

                xml_tag.push_attribute(("v", &*json2str(&v)));

                writer.write_event(XMLEvents::Event::Empty(xml_tag)).unwrap();
            }
        },
        None => { return Err(XMLError::Unknown); }
    };

    writer.write_event(XMLEvents::Event::End(XMLEvents::BytesEnd::borrowed(b"node"))).unwrap();

    osm.nodes.push_str(&*String::from_utf8(writer.into_inner().into_inner()).unwrap());

    Ok(true)
}

pub fn multipoint(_feat: &geojson::Feature, _coords: &Vec<geojson::PointType>, _osm: &mut OSMTypes) {

}

pub fn linestring(feat: &geojson::Feature, coords: &geojson::LineStringType, osm: &mut OSMTypes) -> Result<bool, XMLError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut xml_way = XMLEvents::BytesStart::owned(b"way".to_vec(), 3);

    xml_way.push_attribute(("id", &*feature::get_id(feat).unwrap().to_string()));
    xml_way.push_attribute(("version", &*feature::get_version(feat).unwrap().to_string()));

    writer.write_event(XMLEvents::Event::Start(xml_way)).unwrap();

    let dedupe: HashMap<geojson::PointType, Node>;

    for nd in coords {
        let n_ref: i64;

        if dedupe.has(nd) {
            n_ref = dedupe.get(nd).unwrap();
        } else {
            let node = match add_node(&nd, osm) {
                Ok(node) => node,
                Err(_) => { return Err(XMLError::EncodingFailed); }
            };

            osm.nodes.push_str(&*node.0);

            n_ref = node.1;
        }

        let mut xml_nd = XMLEvents::BytesStart::owned(b"nd".to_vec(), 2);
        xml_nd.push_attribute(("ref", &*n.to_string()));
        writer.write_event(XMLEvents::Event::Empty(xml_nd)).unwrap();
    }

    match *&feat.properties {
        Some(ref props) => {
            for (k, v) in props.iter() {
                let mut xml_tag = XMLEvents::BytesStart::owned(b"tag".to_vec(), 3);
                xml_tag.push_attribute(("k", k.as_str()));

                xml_tag.push_attribute(("v", &*json2str(&v)));
                writer.write_event(XMLEvents::Event::Empty(xml_tag)).unwrap();
            }
        },
        None => { return Err(XMLError::Unknown); }
    };

    writer.write_event(XMLEvents::Event::End(XMLEvents::BytesEnd::borrowed(b"way"))).unwrap();

    osm.ways.push_str(&*String::from_utf8(writer.into_inner().into_inner()).unwrap());

    Ok(true)
}
pub fn multilinestring(_feat: &geojson::Feature, _coords: &Vec<geojson::LineStringType>, _osm: &mut OSMTypes) {

}

pub fn polygon(feat: &geojson::Feature, coords: &geojson::PolygonType, osm: &mut OSMTypes) -> Result<bool, XMLError> {
    if coords.len() == 1 {
        let coords = vec!(coords[0].clone());

        return Ok(linestring(&feat, &coords[0], osm)?);
    }

    //Handle polygons with inners as relations
    Ok(false)
}

pub fn multipolygon(_feat: &geojson::Feature, _coords: &Vec<geojson::PolygonType>, _osm: &mut OSMTypes) {

}

pub fn json2str(v: &serde_json::value::Value) -> String {
    match *v {
        serde_json::value::Value::Null => String::from(""),
        serde_json::value::Value::Bool(ref json_bool) => match *json_bool {
            true => String::from("yes"),
            false => String::from("no")
        },
        serde_json::value::Value::Number(ref json_num) => String::from(format!("{}", json_num)),
        serde_json::value::Value::String(ref json_str) => json_str.to_string(),
        _ => v.to_string()
    }
}

pub fn add_node(coords: &geojson::PointType, osm: &mut OSMTypes) -> Result<(String, i64), XMLError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut xml_node = XMLEvents::BytesStart::owned(b"node".to_vec(), 4);

    osm.node_it = osm.node_it + 1;

    xml_node.push_attribute(("id", &*osm.node_it.to_string()));
    xml_node.push_attribute(("version", "1"));
    xml_node.push_attribute(("lat", &*coords[1].to_string()));
    xml_node.push_attribute(("lon", &*coords[0].to_string()));

    writer.write_event(XMLEvents::Event::Empty(xml_node)).unwrap();

    Ok((String::from_utf8(writer.into_inner().into_inner()).unwrap(), osm.node_it))
}

pub fn parse_osm(xml_node: &XMLEvents::BytesStart, meta: &mut HashMap<String, String>) -> Result<bool, XMLError> {
    for attr in xml_node.attributes() {
        let attr = attr?;

        let key = String::from_utf8_lossy(attr.key);
        let val = String::from_utf8_lossy(attr.value);

        meta.insert(key.to_string(), val.to_string());
    }

    if !meta.contains_key("version") { return Err(XMLError::InternalError(String::from("version required"))); }

    let v: f32 = match meta.get("version") {
        Some(ver) => ver.parse()?,
        None => { return Err(XMLError::InternalError(String::from("version required"))); }
    };

    if v != 0.6 { return Err(XMLError::InternalError(String::from("api only supports 0.6"))); }

    return Ok(true);
}

pub fn parse_node(xml_node: &XMLEvents::BytesStart) -> Result<Node, XMLError> {
    let mut node = Node::new();

    for attr in xml_node.attributes() {
        let attr = attr?;

        match attr.key {
            b"id" => node.id = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"lat" => node.lat = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"lon" => node.lon = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"user" => node.user = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"uid" => node.uid = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"version" => node.version = Some(String::from_utf8_lossy(attr.value).parse()?),
            _ => ()
        }
    }

    return Ok(node);
}

pub fn parse_way(xml_node: &XMLEvents::BytesStart) -> Result<Way, XMLError> {
    let mut way = Way::new();

    for attr in xml_node.attributes() {
        let attr = attr?;

        match attr.key {
            b"id" => way.id = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"version"  => way.version = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"user" => way.user = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"uid" => way.uid = Some(String::from_utf8_lossy(attr.value).parse()?),
            _ => ()
        }
    }

    return Ok(way);
}

pub fn parse_rel(xml_node: &XMLEvents::BytesStart) -> Result<Rel, XMLError> {
    let mut rel = Rel::new();

    for attr in xml_node.attributes() {
        let attr = attr?;

        match attr.key {
            b"id" => rel.id = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"version"  => rel.version = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"user" => rel.user = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"uid" => rel.uid = Some(String::from_utf8_lossy(attr.value).parse()?),
            _ => ()
        }
    }

    return Ok(rel);
}

pub fn parse_nd(xml_node: &XMLEvents::BytesStart) -> Result<i64, XMLError> {
    let mut ndref: Option<i64> = None;

    for attr in xml_node.attributes() {
        let attr = attr?;

        match attr.key {
            b"ref" => ndref = Some(String::from_utf8_lossy(attr.value).parse()?),
            _ => ()
        }
    }

    match ndref {
        Some(val) => Ok(val),
        None => Err(XMLError::InternalError(String::from("unable to parse ndref")))
    }
}

pub fn parse_tag(xml_node: &XMLEvents::BytesStart) -> Result<(String, String), XMLError> {
    let mut k: Option<String> = None;
    let mut v: Option<String> = None;

    for attr in xml_node.attributes() {
        let attr = attr?;

        match attr.key {
            b"k" => k = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"v" => v = Some(String::from_utf8_lossy(attr.value).parse()?),
            _ => ()
        }
    }

    return Ok((match k {
        Some(key) => key,
        None => { return Err(XMLError::InternalError(String::from("unable to parse key"))) }
    }, match v {
        Some(val) => val,
        None => { return Err(XMLError::InternalError(String::from("unable to parse value"))) }
    }));
}

pub fn parse_member(xml_node: &XMLEvents::BytesStart) -> Result<(Option<Value>, Option<i64>, Option<String>), XMLError> {
    let mut rtype: Option<Value> = None;
    let mut rref: Option<i64> = None;
    let mut rrole: Option<String> = None;

    for attr in xml_node.attributes() {
        let attr = attr?;

        match attr.key {
            b"type" => rtype = Some(match attr.value {
                b"node" => Value::Node,
                b"way" => Value::Way,
                b"relation" => Value::Rel,
                _ => { return Err(XMLError::InternalError(String::from("invalid type"))); }
            }),
            b"rref" => rref = Some(String::from_utf8_lossy(attr.value).parse()?),
            b"rrole" => rrole = Some(String::from_utf8_lossy(attr.value).parse()?),
            _ => ()
        }
    }

    return Ok((rtype, rref, rrole))
}
