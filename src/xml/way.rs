use std::fmt;
use std::collections::HashMap;
use xml::*;

pub struct Way {
    pub id: Option<i64>,
    pub user: Option<String>,
    pub uid: Option<i32>,
    pub modified: bool,
    pub nodes: Vec<i64>,
    pub tags: HashMap<String, String>,
    pub version: Option<i32>,
    pub parents: Vec<i64>
}

impl Generic for Way {
    fn new() -> Way {
        Way {
            id: None,
            tags: HashMap::new(),
            modified: false,
            user: None,
            uid: None,
            nodes: Vec::new(),
            parents: Vec::new(),
            version: None
        }
    }

    fn value(&self) -> Value {
        Value::Way
    }

    fn set_tag(&mut self, k: String, v: String) {
        self.tags.insert(k, v);
    }

    fn is_valid(&self) -> bool {
        match self.id {
            None => { return false; },
            Some(_) => ()
        }
        match self.version {
            None => { return false; },
            Some(_) => ()
        }

        if self.nodes.len() == 0 {
            return false;
        }

        return true;
    }
}

impl fmt::Display for Way {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Way: id={}]", self.id.unwrap())
    }
}

