use std::fmt;
use std::collections::HashMap;
use xml::*;

pub struct Node {
    pub id: Option<i64>,
    pub lat: Option<f32>,
    pub user: Option<String>,
    pub action: Option<Action>,
    pub uid: Option<i32>,
    pub lon: Option<f32>,
    pub modified: bool,
    pub tags: serde_json::Map<String, serde_json::Value>,
    pub version: Option<i32>,
    pub parents: Vec<i64>
}

impl Generic for Node {
    fn new() -> Node {
        Node {
            id: None,
            lat: None,
            lon: None,
            user: None,
            uid: None,
            modified: false,
            action: None,
            tags: serde_json::Map::new(),
            parents: Vec::new(),
            version: None
        }
    }

    fn value(&self) -> Value {
        Value::Node
    }

    fn set_tag(&mut self, k: String, v: String) {
        self.tags.insert(k, serde_json::Value::String(v));
    }

    fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    fn to_feat(&self, tree: &OSMTree) -> geojson::Feature {
        let mut foreign = serde_json::Map::new();

        foreign.insert(String::from("action"), serde_json::Value::String(match self.action {
            Some(Action::Create) => String::from("create"),
            Some(Action::Modify) => String::from("modify"),
            Some(Action::Delete) => String::from("delete"),
            _ => String::new()
        }));

        foreign.insert(String::from("version"), json!(self.version));

        let mut coords: Vec<f64> = Vec::new();
        coords.push(self.lon.unwrap() as f64);
        coords.push(self.lat.unwrap() as f64);

        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::new(
                geojson::Value::Point(coords)
            )),
            id: Some(json!(self.id.clone())),
            properties: Some(self.tags.clone()),
            foreign_members: Some(foreign)
        }
    }

    fn is_valid(&self) -> bool {
        match self.id {
            None => { return false; },
            Some(_) => ()
        }
        match self.lat {
            None => { return false; },
            Some(_) => ()
        }
        match self.lon {
            None => { return false; },
            Some(_) => ()
        }
        match self.version {
            None => { return false; },
            Some(_) => ()
        }

        return true;
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Node: id={}]", self.id.unwrap())
    }
}
