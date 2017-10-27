extern crate geojson;

pub enum XMLError {
    Unknown
}

impl XMLError {
    pub fn to_string(&self) -> &str {
        match &self {
            Unknown => {
                "Unknown Error"
            }
        }
    }
}

pub fn from(geojson: &geojson::FeatureCollection) -> Result<String, XMLError> {

    let test_str = String::from("test");
    Ok(test_str)
}
