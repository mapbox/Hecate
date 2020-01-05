use crate::err::HecateError;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub enum Scope {
    Read,
    Full
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        match self {
            Scope::Read => String::from("read"),
            Scope::Full => String::from("full")
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Token {
    pub name: String,
    pub uid: i64,
    pub token: String,
    pub expiry: Option<String>,
    pub scope: Scope
}

impl Token {
    pub fn new(name: String, uid: i64, token: String, expiry: Option<String>, scope: Scope) -> Self {
        Token {
            name,
            uid,
            token,
            expiry,
            scope
        }
    }

    pub fn create(conn: &impl postgres::GenericConnection, name: impl ToString, uid: i64, hours: Option<i64>, scope: Scope) -> Result<Self, HecateError> {
        let hours: Option<String> = match hours {
            None => None,
            Some(hours) => Some(format!("{} hours", hours))
        };

        match conn.query("
            INSERT INTO users_tokens (name, uid, token, expiry, scope)
                VALUES (
                    $1,
                    $2,
                    md5(random()::TEXT),
                    CASE
                        WHEN $3 IS NULL THEN NULL
                        ELSE now() + ($3::TEXT)::INTERVAL
                    END,
                    $4
                )
                RETURNING
                    name,
                    uid,
                    token,
                    expiry::TEXT
        ", &[ &name.to_string(), &uid, &hours, &scope.to_string() ]) {
            Ok(res) => {
                let name: String = res.get(0).get(0);
                let uid: i64 = res.get(0).get(1);
                let token: String = res.get(0).get(2);
                let expiry: Option<String> = res.get(0).get(3);

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
                let expiry: Option<String> = res.get(0).get(3);
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

pub fn list(conn: &impl postgres::GenericConnection, uid: i64) -> Result<Vec<Token>, HecateError> {
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
            uid = $1
    ", &[ &uid ]) {
        Ok(rows) => {
            let mut tokens = Vec::with_capacity(rows.len());

            for row in rows.iter() {
                let name: String = row.get(0);
                let uid: i64 = row.get(1);
                let token: String = row.get(2);
                let expiry: Option<String> = row.get(3);
                let scope: String = row.get(4);

                let scope = match scope.as_str() {
                    "full" => Scope::Full,
                    _ => Scope::Read
                };

                tokens.push(Token::new(name, uid, token, expiry, scope))
            }

            Ok(tokens)

        },
        Err(_) => Err(HecateError::new(404, String::from("Tokens Not Found"), None))
    }

}
