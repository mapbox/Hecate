#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Meta {
    pub key: String,
    pub value: serde_json::Value,
}

