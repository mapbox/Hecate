use postgres;
use geo::prelude::*;
use crate::err::HecateError;
use serde_json::Value;

pub struct WebHook {
    id: i64,
    name: String,
    actions: Vec<String>,
    url: String
}

impl WebHook {
    pub fn new(id: i64, name: String, actions: Vec<String>, url: String) -> Self {
        WebHook {
            id: id,
            name: name,
            actions: actions,
            url: url
        }
    }
}

pub fn list(conn: &impl postgres::GenericConnection) -> Result<Vec<WebHook>, HecateError> {
    match conn.query("
        SELECT
            id,
            name,
            actions,
            url
        FROM
            webhooks
    ", &[]) {
        Ok(results) => {
            let mut hooks: Vec<WebHook> = Vec::with_capacity(results.len());

            for result in results.iter() {
                WebHook::new(result.get(0), result.get(1), result.get(2), result.get(3));
            }

            Ok(hooks)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}
