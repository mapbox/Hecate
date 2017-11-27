use std::fmt;
use xml::*;

pub struct Way {
    pub id: Option<i64>,
    pub user: Option<String>,
    pub uid: Option<i32>,
    pub modified: bool,
    pub nodes: Vec<i64>,
    pub action: Option<Action>,
    pub tags: serde_json::Map<String, serde_json::Value>,
    pub version: Option<i32>,
    pub parents: Vec<i64>
}

impl Generic for Way {
    fn new() -> Way {
        Way {
            id: None,
            tags: serde_json::Map::new(),
            modified: false,
            user: None,
            uid: None,
            nodes: Vec::new(),
            action: None,
            parents: Vec::new(),
            version: None
        }
    }

    fn value(&self) -> Value {
        Value::Way
    }

    fn set_tag(&mut self, k: String, v: String) {
        self.tags.insert(k, serde_json::Value::String(v));
    }

    fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    fn to_feat(&self, tree: &OSMTree) -> Result<geojson::Feature, XMLError> {
        let mut foreign = serde_json::Map::new();

        match self.is_valid() {
            Err(err) => { return Err(XMLError::InvalidWay(err)); },
            _ => ()
        }

        foreign.insert(String::from("action"), serde_json::Value::String(match self.action {
            Some(Action::Create) => String::from("create"),
            Some(Action::Modify) => String::from("modify"),
            Some(Action::Delete) => String::from("delete"),
            _ => { return Err(XMLError::InvalidNode(String::from("Missing or invalid action"))); }
        }));

        foreign.insert(String::from("version"), json!(self.version));

        let mut linecoords: Vec<geojson::Position> = Vec::new();

        for nid in &self.nodes {
            let node = tree.get_node(&nid).unwrap();

            let mut coords: Vec<f64> = Vec::new();
            coords.push(node.lon.unwrap() as f64);
            coords.push(node.lat.unwrap() as f64);

            linecoords.push(coords);
        }

        if self.nodes[0] == self.nodes[self.nodes.len() - 1] {
            //Handle Polygons

            let mut polycoords: Vec<Vec<geojson::Position>> = Vec::new();
            polycoords.push(linecoords);

            Ok(geojson::Feature {
                bbox: None,
                geometry: Some(geojson::Geometry::new(
                    geojson::Value::Polygon(polycoords)
                )),
                id: Some(json!(self.id.clone())),
                properties: Some(self.tags.clone()),
                foreign_members: Some(foreign)
            })
        } else {
            //Handle LineStrings

            Ok(geojson::Feature {
                bbox: None,
                geometry: Some(geojson::Geometry::new(
                    geojson::Value::LineString(linecoords)
                )),
                id: Some(json!(self.id.clone())),
                properties: Some(self.tags.clone()),
                foreign_members: Some(foreign)
            })
        }
    }

    fn is_valid(&self) -> Result<bool, String> {
        match self.id {
            None => { return Err(String::from("Missing id")); },
            Some(_) => ()
        }
        match self.version {
            None => { return Err(String::from("Missing version")); },
            Some(_) => ()
        }

        if self.nodes.len() == 0 {
            return Err(String::from("Node references cannot be empty"));
        }

        return Ok(true);
    }
}

impl fmt::Display for Way {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Way: id={}]", self.id.unwrap())
    }
}

