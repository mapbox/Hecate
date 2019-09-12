use crate::err::HecateError;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Scope {
    Full,
    Osm
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Token {
    pub name: String,
    pub scope: Scope,
    pub uid: i64,
    pub token: String,
    pub expiry: String
}

impl Token {
    pub fn new(name: String, uid: i64, token: String, expiry: String, scope: Scope) -> Self {
        Token {
            name: name,
            uid: uid,
            token: token,
            expiry: expiry,
            scope: scope
        }
    }

    pub fn create(conn: &impl postgres::GenericConnection, name: impl ToString, uid: &i64, hours: &i64, scope: Scope) -> Result<Self, HecateError> {
        if hours > &16 {
            return Err(HecateError::new(400, String::from("Token Expiry Cannot Exceed 16 hours"), None));
        }

        let scope_str = match scope {
            Scope::Full => "Full",
            Scope::Osm => "Osm"
        };

        match conn.query("
            INSERT INTO users_tokens (name, uid, scope, token, expiry)
                VALUES (
                    $1,
                    $2,
                    $3,
                    md5(random()::TEXT),
                    now() + INTERVAL '$4 hours'
                )
                RETURNING
                    name,
                    uid,
                    token,
                    expiry::TEXT
        ", &[ &name.to_string(), &uid, &scope_str, &hours ]) {
            Ok(res) => {
                let name: String = res.get(0).get(0);
                let uid: i64 = res.get(0).get(1);
                let token: String = res.get(0).get(2);
                let expiry: String = res.get(0).get(3);

                Ok(Token::new(name, uid, token, expiry, scope))
            },
            Err(err) => Err(HecateError::from_db(err))
        }

    }

    pub fn get(conn: &impl postgres::GenericConnection, uid: &i64, token: &String) -> Result<Self, HecateError> {
        match conn.query("
            SELECT row_to_json(t)
            FROM (
                SELECT
                    name,
                    uid,
                    token,
                    expiry
                FROM
                    users_tokens
                WHERE
                    uid = $1,
                    token = $2
            ) t
        ", &[ &uid, &token ]) {
            Ok(res) => {
                let token: serde_json::Value = res.get(0).get(0);
                let token: Token = serde_json::from_value(token).unwrap();
                Ok(token)
            },
            Err(err) => Err(HecateError::from_db(err))
        }

    }
}

pub fn destroy(conn: &impl postgres::GenericConnection, uid: &i64, token: &String) -> Result<bool, HecateError> {
    match conn.query("
        DELETE FROM users_tokens
            WHERE
                token = $1
                AND uid = $2;
    ", &[ &token, &uid ]) {
        Ok(_) => Ok(true),
        Err(_) => Err(HecateError::new(404, String::from("Token Not Found"), None))
    }
}
