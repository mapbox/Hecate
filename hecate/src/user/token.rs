use crate::err::HecateError;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub enum Scope {
    Read,
    Full
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Token {
    pub name: String,
    pub uid: i64,
    pub token: String,
    pub expiry: String,
    pub scope: Scope
}

impl Token {
    pub fn new(name: String, uid: i64, token: String, expiry: String, scope: Scope) -> Self {
        Token {
            name,
            uid,
            token,
            expiry,
            scope
        }
    }

    pub fn create(conn: &impl postgres::GenericConnection, name: impl ToString, uid: i64, hours: i64, scope: Scope) -> Result<Self, HecateError> {
        if hours > 336 {
            return Err(HecateError::new(400, String::from("Token Expiry Cannot Exceed 2 weeks (336 hours)"), None));
        }

        let hours = format!("{} hours", hours);

        let scope_str = match scope {
            Scope::Full => "full",
            Scope::Read => "read"
        };

        match conn.query("
            INSERT INTO users_tokens (name, uid, token, expiry, scope)
                VALUES (
                    $1,
                    $2,
                    md5(random()::TEXT),
                    now() + ($3::TEXT)::INTERVAL,
                    $4
                )
                RETURNING
                    name,
                    uid,
                    token,
                    expiry::TEXT
        ", &[ &name.to_string(), &uid, &hours, &scope_str ]) {
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

    pub fn get(conn: &impl postgres::GenericConnection, uid: i64, token: &str) -> Result<Self, HecateError> {
        match conn.query("
            SELECT
                name,
                uid,
                token,
                expiry::TEXT,
                scope
            FROM
                users_tokens
            WHERE
                uid = $1,
                token = $2
        ", &[ &uid, &token ]) {
            Ok(res) => {
                let name: String = res.get(0).get(0);
                let uid: i64 = res.get(0).get(1);
                let token: String = res.get(0).get(2);
                let expiry: String = res.get(0).get(3);
                let scope: String = res.get(0).get(4);

                let scope = match scope.as_str() {
                    "full" => Scope::Full,
                    _ => Scope::Read
                };

                Ok(Token::new(name, uid, token, expiry, scope))
            },
            Err(err) => Err(HecateError::from_db(err))
        }

    }
}

pub fn destroy(conn: &impl postgres::GenericConnection, uid: i64, token: &str) -> Result<bool, HecateError> {
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
