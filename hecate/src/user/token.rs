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
    pub id: Option<String>,
    pub name: String,
    pub uid: i64,
    pub token: Option<String>,
    pub expiry: Option<String>,
    pub scope: Scope
}

impl Token {
    pub fn new(id: Option<String>, name: String, uid: i64, token: Option<String>, expiry: Option<String>, scope: Scope) -> Self {
        Token {
            id,
            name,
            uid,
            token,
            expiry,
            scope
        }
    }

    pub fn token(&self) -> Result<String, HecateError> {
        match &self.token {
            None => Err(HecateError::new(500, String::from("Could not retrieve token"), None)),
            Some(token) => Ok(token.to_string())
        }
    }

    pub fn create(conn: &impl postgres::GenericConnection, name: impl ToString, uid: i64, hours: Option<i64>, scope: Scope) -> Result<Self, HecateError> {
        let res = match hours {
            None => match conn.query("
                INSERT INTO users_tokens (id, name, uid, token, scope)
                    VALUES (
                        uuid_generate_v4(),
                        $1,
                        $2,
                        md5(random()::TEXT),
                        $3
                    )
                    RETURNING
                        id::TEXT,
                        name,
                        uid,
                        token,
                        expiry::TEXT
            ", &[ &name.to_string(), &uid, &scope.to_string() ]) {
                Err(err) => { return Err(HecateError::from_db(err)); },
                Ok(res) => res
            },
            Some(hours) => {
                let hours = format!("{} hours", hours);

                match conn.query("
                    INSERT INTO users_tokens (id, name, uid, token, expiry, scope)
                        VALUES (
                            uuid_generate_v4(),
                            $1,
                            $2,
                            md5(random()::TEXT),
                            {hours}
                            $4
                        )
                        RETURNING
                            id::TEXT,
                            name,
                            uid,
                            token,
                            expiry::TEXT
                ", &[ &name.to_string(), &uid, &hours, &scope.to_string() ]) {
                    Err(err) => { return Err(HecateError::from_db(err)); },
                    Ok(res) => res
                }
            }
        };

        let id: String = res.get(0).get(0);
        let name: String = res.get(0).get(1);
        let uid: i64 = res.get(0).get(2);
        let token: String = res.get(0).get(3);
        let expiry: Option<String> = res.get(0).get(4);

        Ok(Token::new(Some(id), name, uid, Some(token), expiry, scope))
    }

    pub fn get(conn: &impl postgres::GenericConnection, uid: i64, token: &str) -> Result<Self, HecateError> {
        match conn.query("
            SELECT
                id::TEXT,
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
                let id: String = res.get(0).get(0);
                let name: String = res.get(0).get(1);
                let uid: i64 = res.get(0).get(2);
                let token: String = res.get(0).get(3);
                let expiry: Option<String> = res.get(0).get(4);
                let scope: String = res.get(0).get(5);

                let scope = match scope.as_str() {
                    "full" => Scope::Full,
                    _ => Scope::Read
                };

                Ok(Token::new(Some(id), name, uid, Some(token), expiry, scope))
            },
            Err(err) => Err(HecateError::from_db(err))
        }

    }
}

pub fn destroy(conn: &impl postgres::GenericConnection, uid: i64, token: &str) -> Result<bool, HecateError> {
    if token.contains('-') {
        match conn.query("
            DELETE FROM users_tokens
                WHERE
                    id::TEXT = $1
                    AND uid = $2;
        ", &[ &token, &uid ]) {
            Ok(_) => Ok(true),
            Err(err) => Err(HecateError::new(404, String::from("Token Not Found"), Some(err.to_string())))
        }
    } else {
        match conn.query("
            DELETE FROM users_tokens
                WHERE
                    token = $1
                    AND uid = $2;
        ", &[ &token, &uid ]) {
            Ok(_) => Ok(true),
            Err(err) => Err(HecateError::new(404, String::from("Token Not Found"), Some(err.to_string())))
        }
    }
}

pub fn list(conn: &impl postgres::GenericConnection, uid: i64) -> Result<Vec<Token>, HecateError> {
    match conn.query("
        SELECT
            id::TEXT,
            name,
            uid,
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
                let id: String = row.get(0);
                let name: String = row.get(1);
                let uid: i64 = row.get(2);
                let expiry: Option<String> = row.get(3);
                let scope: String = row.get(4);

                let scope = match scope.as_str() {
                    "full" => Scope::Full,
                    _ => Scope::Read
                };

                tokens.push(Token::new(Some(id), name, uid, None, expiry, scope))
            }

            Ok(tokens)

        },
        Err(_) => Err(HecateError::new(404, String::from("Tokens Not Found"), None))
    }

}
