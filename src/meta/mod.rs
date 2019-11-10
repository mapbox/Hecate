use crate::err::HecateError;
use serde_json::Value;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Meta {
    pub key: String,
    pub value: Value
}

impl Meta {
    pub fn new(key: String, value: Value) -> Self {
        Meta {
            key: key,
            value: value
        }
    }

    pub fn get(conn: &impl postgres::GenericConnection, key: impl ToString) -> Result<Self, HecateError> {
        let key = key.to_string();

        match conn.query("
            SELECT value::JSON FROM meta WHERE key = $1;
        ", &[&key]) {
            Ok(rows) => {
                if rows.len() == 0 {
                    Ok(Meta::new(key, json!(false)))
                } else {
                    Ok(Meta::new(key, rows.get(0).get(0)))
                }
            },
            Err(err) => Err(HecateError::from_db(err))
        }
    }

    pub fn set(&self, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        match conn.query("
            INSERT INTO meta (key, value) VALUES ($1, $2)
                ON CONFLICT (key) DO
                    UPDATE
                        SET value = $2
                        WHERE meta.key = $1
        ", &[ &self.key, &self.value ]) {
            Ok(_) => Ok(true),
            Err(err) => Err(HecateError::from_db(err))
        }
    }

}

pub fn list(conn: &impl postgres::GenericConnection) -> Result<Vec<String>, HecateError> {
    match conn.query("
        SELECT key FROM meta ORDER BY key
    ", &[]) {
        Ok(rows) => {
            let mut names = Vec::<String>::new();

            for row in rows.iter() {
                names.push(row.get(0));
            }

            Ok(names)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn delete(conn: &impl postgres::GenericConnection, key: &String) -> Result<bool, HecateError> {
    match conn.query("
        DELETE FROM meta WHERE key = $1
    ", &[ &key ]) {
        Ok(_) => Ok(true),
        Err(err) => Err(HecateError::from_db(err))
    }
}
