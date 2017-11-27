use std::fmt;
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

    fn to_feat(&self, tree: &OSMTree) -> Result<geojson::Feature, XMLError> {
        let mut foreign = serde_json::Map::new();

        match self.is_valid() {
            Err(err) => { return Err(XMLError::InvalidNode(err)); },
            _ => ()
        }

        foreign.insert(String::from("action"), serde_json::Value::String(match self.action {
            Some(Action::Create) => String::from("create"),
            Some(Action::Modify) => String::from("modify"),
            Some(Action::Delete) => String::from("delete"),
            _ => { return Err(XMLError::InvalidNode(String::from("Missing or invalid action"))); }
        }));

        foreign.insert(String::from("version"), json!(self.version));

        let mut coords: Vec<f64> = Vec::new();
        coords.push(self.lon.unwrap() as f64);
        coords.push(self.lat.unwrap() as f64);

        Ok(geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::new(
                geojson::Value::Point(coords)
            )),
            id: Some(json!(self.id.clone())),
            properties: Some(self.tags.clone()),
            foreign_members: Some(foreign)
        })
    }

    fn is_valid(&self) -> Result<bool, String> {
        match self.id {
            None => { return Err(String::from("Missing id")) },
            Some(_) => ()
        }
        match self.lat {
            None => { return Err(String::from("Missing lat")); },
            Some(_) => ()
        }
        match self.lon {
            None => { return Err(String::from("Missing lon")); },
            Some(_) => ()
        }
        match self.version {
            None => { return Err(String::from("Missing version")); },
            Some(_) => ()
        }

        return Ok(true);
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Node: id={}]", match self.id {
            None => String::from("None"),
            Some(ref id) => id.to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut n = Node::new();
        assert_eq!(n.id, None);
        assert_eq!(n.lat, None);
        assert_eq!(n.lon, None);
        assert_eq!(n.user, None);
        assert_eq!(n.uid, None);
        assert_eq!(n.modified, false);
        assert_eq!(n.action, None);
        assert_eq!(n.tags.is_empty(), true);
        assert_eq!(n.parents.len(), 0);
        assert_eq!(n.version, None);

        assert_eq!(n.value(), Value::Node);
        assert_eq!(n.is_valid().is_err(), true);

        assert_eq!(format!("{}", n), "[Node: id=None]");

        n.id = Some(1);
        assert_eq!(format!("{}", n), "[Node: id=1]");

        n.id = Some(-1);
        assert_eq!(format!("{}", n), "[Node: id=-1]");
    }

    #[test]
    fn tags() {
        let mut n = Node::new();
        assert_eq!(n.has_tags(), false);
        n.set_tag(String::from("hello"), String::from("world"));
        assert_eq!(n.has_tags(), true);

        assert_eq!(format!("{}", n), "[Node: id=None]");
    }

    #[test]
    fn validity() {
        let mut n = Node::new();
        assert_eq!(n.is_valid().is_err(), true);
        n.id = Some(1);
        assert_eq!(n.is_valid().is_err(), true);
        n.lat = Some(1.1);
        assert_eq!(n.is_valid().is_err(), true);
        n.lon = Some(2.2);
        assert_eq!(n.is_valid().is_err(), true);
        n.version = Some(1);
        assert_eq!(n.is_valid().is_err(), false);
    }

    #[test]
    fn to_feat() {
        let mut n = Node::new();
        let tree = OSMTree::new();
        assert_eq!(n.to_feat(&tree).err(), Some(XMLError::InvalidNode(String::from("Missing id"))));

        n.id = Some(1);
        assert_eq!(n.to_feat(&tree).err(), Some(XMLError::InvalidNode(String::from("Missing lat"))));

        n.lat = Some(1.1);
        assert_eq!(n.to_feat(&tree).err(), Some(XMLError::InvalidNode(String::from("Missing lon"))));

        n.lon = Some(2.2);
        assert_eq!(n.to_feat(&tree).err(), Some(XMLError::InvalidNode(String::from("Missing version"))));

        n.version = Some(1);
        assert_eq!(n.to_feat(&tree).err(), Some(XMLError::InvalidNode(String::from("Missing or invalid action"))));

        n.action = Some(Action::Create);

        let mut fmem = serde_json::Map::new();
        fmem.insert(String::from("action"), json!(String::from("create")));
        fmem.insert(String::from("version"), json!(1));

        assert_eq!(n.to_feat(&tree).ok(), Some(geojson::Feature {
            bbox: None,
            id: Some(json!(1)),
            properties: Some(serde_json::Map::new()),
            geometry: Some(geojson::Geometry::new(geojson::Value::Point(vec!(2.200000047683716, 1.100000023841858)))),
            foreign_members: Some(fmem.clone())
        }));

        n.action = Some(Action::Modify);
        fmem.insert(String::from("action"), json!(String::from("modify")));
        assert_eq!(n.to_feat(&tree).ok(), Some(geojson::Feature {
            bbox: None,
            id: Some(json!(1)),
            properties: Some(serde_json::Map::new()),
            geometry: Some(geojson::Geometry::new(geojson::Value::Point(vec!(2.200000047683716, 1.100000023841858)))),
            foreign_members: Some(fmem.clone())
        }));

        n.action = Some(Action::Delete);
        fmem.insert(String::from("action"), json!(String::from("delete")));
        assert_eq!(n.to_feat(&tree).ok(), Some(geojson::Feature {
            bbox: None,
            id: Some(json!(1)),
            properties: Some(serde_json::Map::new()),
            geometry: Some(geojson::Geometry::new(geojson::Value::Point(vec!(2.200000047683716, 1.100000023841858)))),
            foreign_members: Some(fmem.clone())
        }));
    }
}
