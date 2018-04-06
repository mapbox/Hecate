use std::fmt;
use xml::*;

#[derive(Debug, Clone, PartialEq)]
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
        let value = match serde_json::from_str::<serde_json::Value>(&*v) {
            Ok(value) => value,
            Err(_) => serde_json::Value::String(v)
        };
        self.tags.insert(k, value);
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
            _ => { return Err(XMLError::InvalidWay(String::from("Missing or invalid action"))); }
        }));

        foreign.insert(String::from("version"), json!(self.version));

        let mut linecoords: Vec<geojson::Position> = Vec::new();

        for nid in &self.nodes {
            let node = match tree.get_node(&nid) {
                Err(_) => { return Err(XMLError::InvalidWay(String::from("Node reference not found in tree"))); },
                Ok(n) => n
            };

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
        write!(f, "[Way: id={}]", match self.id {
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
        let mut w = Way::new();
        assert_eq!(w.id, None);
        assert_eq!(w.tags, serde_json::Map::new());
        assert_eq!(w.modified, false);
        assert_eq!(w.user, None);
        assert_eq!(w.uid, None);
        assert_eq!(w.nodes, Vec::<i64>::new());
        assert_eq!(w.action, None);
        assert_eq!(w.parents, Vec::<i64>::new());
        assert_eq!(w.version, None);

        assert_eq!(w.value(), Value::Way);
        assert_eq!(w.is_valid().is_err(), true);

        assert_eq!(format!("{}", w), "[Way: id=None]");

        w.id = Some(1);
        assert_eq!(format!("{}", w), "[Way: id=1]");

        w.id = Some(-1);
        assert_eq!(format!("{}", w), "[Way: id=-1]");
    }

    #[test]
    fn tags() {
        let mut w = Way::new();
        assert_eq!(w.has_tags(), false);
        w.set_tag(String::from("hello"), String::from("world"));
        assert_eq!(w.has_tags(), true);

        assert_eq!(format!("{}", w), "[Way: id=None]");
    }

    #[test]
    fn validity() {
        let mut w = Way::new();
        assert_eq!(w.is_valid().is_err(), true);
        w.id = Some(1);
        assert_eq!(w.is_valid().is_err(), true);
        w.version = Some(1);
        assert_eq!(w.is_valid().is_err(), true);
        w.nodes.push(1);
        assert_eq!(w.is_valid().is_err(), false);
    }

    #[test]
    fn to_feat() {
        let mut w = Way::new();
        let mut tree = OSMTree::new();
        assert_eq!(w.to_feat(&tree).err(), Some(XMLError::InvalidWay(String::from("Missing id"))));

        w.id = Some(1);
        assert_eq!(w.to_feat(&tree).err(), Some(XMLError::InvalidWay(String::from("Missing version"))));

        w.version = Some(1);
        assert_eq!(w.to_feat(&tree).err(), Some(XMLError::InvalidWay(String::from("Node references cannot be empty"))));

        let mut n1 = Node::new();
        let mut n2 = Node::new();
        let mut n3 = Node::new();

        n1.id = Some(1);
        n2.id = Some(2);
        n3.id = Some(3);
        n1.lat = Some(1.1);
        n2.lat = Some(2.2);
        n3.lat = Some(3.3);
        n1.lon = Some(1.1);
        n2.lon = Some(2.2);
        n3.lon = Some(3.3);
        n1.version = Some(1);
        n2.version = Some(1);
        n3.version = Some(1);

        assert_eq!(n1.is_valid(), Ok(true));
        assert_eq!(n2.is_valid(), Ok(true));

        // Add node refs to way but don't add them to tree
        w.nodes.push(1);
        w.nodes.push(2);

        assert_eq!(w.to_feat(&tree).err(), Some(XMLError::InvalidWay(String::from("Missing or invalid action"))));

        w.action = Some(Action::Create);
        assert_eq!(w.to_feat(&tree).err(), Some(XMLError::InvalidWay(String::from("Node reference not found in tree"))));

        assert_eq!(tree.add_node(n1.clone()), Ok(true));
        assert_eq!(tree.add_node(n2.clone()), Ok(true));
        assert_eq!(tree.add_node(n3.clone()), Ok(true));

        assert_eq!(tree.get_node(&2), Ok(&n2));
        assert_eq!(tree.get_node(&2), Ok(&n2));
        assert_eq!(tree.get_node(&3), Ok(&n3));

        let mut fmem = serde_json::Map::new();
        fmem.insert(String::from("action"), json!(String::from("create")));
        fmem.insert(String::from("version"), json!(1));

        let mut coords: Vec<geojson::Position> = Vec::new();
        coords.push(vec!(1.100000023841858, 1.100000023841858));
        coords.push(vec!(2.200000047683716, 2.200000047683716));

        assert_eq!(w.to_feat(&tree).ok(), Some(geojson::Feature {
            bbox: None,
            id: Some(json!(1)),
            properties: Some(serde_json::Map::new()),
            geometry: Some(geojson::Geometry::new(geojson::Value::LineString(coords.clone()))),
            foreign_members: Some(fmem.clone())
        }));

        w.nodes.push(3);
        w.nodes.push(1);

        coords.push(vec!(3.299999952316284, 3.299999952316284));
        coords.push(vec!(1.100000023841858, 1.100000023841858));
        let mut pcoords: Vec<Vec<geojson::Position>> = Vec::new();
        pcoords.push(coords);

        assert_eq!(w.to_feat(&tree).ok(), Some(geojson::Feature {
            bbox: None,
            id: Some(json!(1)),
            properties: Some(serde_json::Map::new()),
            geometry: Some(geojson::Geometry::new(geojson::Value::Polygon(pcoords.clone()))),
            foreign_members: Some(fmem.clone())
        }));

        w.action = Some(Action::Modify);
        fmem.insert(String::from("action"), json!(String::from("modify")));
        assert_eq!(w.to_feat(&tree).ok(), Some(geojson::Feature {
            bbox: None,
            id: Some(json!(1)),
            properties: Some(serde_json::Map::new()),
            geometry: Some(geojson::Geometry::new(geojson::Value::Polygon(pcoords.clone()))),
            foreign_members: Some(fmem.clone())
        }));

        w.action = Some(Action::Delete);
        fmem.insert(String::from("action"), json!(String::from("delete")));
        assert_eq!(w.to_feat(&tree).ok(), Some(geojson::Feature {
            bbox: None,
            id: Some(json!(1)),
            properties: Some(serde_json::Map::new()),
            geometry: Some(geojson::Geometry::new(geojson::Value::Polygon(pcoords.clone()))),
            foreign_members: Some(fmem.clone())
        }));
    }
}
