use crate::err::HecateError;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    password: Option<String>,
    pub email: String,
    pub meta: Option<serde_json::Value>,
    pub access: Option<String>
}

impl User {
    pub fn new(username: String, password: Option<String>, email: String, meta: Option<serde_json::Value>) -> Self {
        User {
            id: None,
            username: username,
            password: password,
            email: email,
            meta: meta,
            access: None
        }
    }

    pub fn to_value(self) -> serde_json::Value {
        json!({
            "id": self.id,
            "username": self.username,
            "email": self.email,
            "meta": self.meta,
            "access": self.access
        })
    }

    pub fn password(&mut self, password: String) {
        self.password = Some(password);
    }

    pub fn is_admin(&self) -> bool {
        if self.access == Some(String::from("admin")) {
            true
        } else {
            false
        }
    }

    pub fn admin(&mut self, admin: bool) {
        match admin {
            true => self.access = Some(String::from("admin")),
            false => self.access = None
        };
    }

    pub fn set(&self, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        if self.id.is_some() {
            match conn.query("
                UPDATE users
                    SET
                        username = $1,
                        email = $2,
                        meta = $3,
                        access = $4
                    WHERE
                        id = $5
            ", &[ &self.username, &self.email, &self.meta, &self.access, &self.id ]) {
                Ok(_) => Ok(true),
                Err(err) => Err(HecateError::from_db(err))
            }

        } else {
            let password = match self.password {
                Some(ref password) => password,
                None => { return Err(HecateError::new(400, String::from("Password must be present to create new user"), None)); }
            };

            match conn.query("
                INSERT INTO users (username, password, email, meta, access)
                    VALUES ($1, crypt($2, gen_salt('bf', 10)), $3, $4, $5);
            ", &[ &self.username, &password, &self.email, &self.meta, &self.access ]) {
                Ok(_) => Ok(true),
                Err(err) => {
                    if err.as_db().is_some() && err.as_db().unwrap().code.code() == "23505" {
                        Err(HecateError::new(400, String::from("User/Email Exists"), None))
                    } else {
                        Err(HecateError::from_db(err))
                    }
                }
            }
        }
    }

    pub fn get(conn: &impl postgres::GenericConnection, uid: &i64) -> Result<Self, HecateError> {
        match conn.query("
            SELECT row_to_json(u)
            FROM (
                SELECT
                    id,
                    username,
                    email,
                    COALESCE(meta, '{}'::JSONB) AS meta,
                    access
                FROM
                    users
                WHERE id = $1
            ) u
        ", &[ &uid ]) {
            Ok(res) => {
                let res: serde_json::Value = res.get(0).get(0);

                let user: User = match serde_json::from_value(res) {
                    Ok(user) => {
                        user
                    },
                    Err(err) => {
                        return Err(HecateError::new(500, String::from("Failed to deserialize user"), Some(err.to_string())));
                    }
                };

                Ok(user)
            },
            Err(err) => Err(HecateError::from_db(err))
        }
    }

}

pub fn list(conn: &impl postgres::GenericConnection, limit: &Option<i16>) -> Result<serde_json::Value, HecateError> {
    let limit: i16 = match limit {
        None => 100,
        Some(limit) => if *limit > 100 { 100 } else { *limit }
    };

    match conn.query("
        SELECT
            COALESCE(json_agg(row_to_json(row)), '[]'::JSON)
        FROM (
            SELECT
                id,
                access,
                username
            FROM
                users
            ORDER BY
                username
            LIMIT $1::SmallInt
        ) row;
    ", &[ &limit ]) {
        Ok(rows) => Ok(rows.get(0).get(0)),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn filter(conn: &impl postgres::GenericConnection, filter: &String, limit: &Option<i16>) -> Result<serde_json::Value, HecateError> {
    let limit: i16 = match limit {
        None => 100,
        Some(limit) => if *limit > 100 { 100 } else { *limit }
    };

    match conn.query("
        SELECT
            COALESCE(json_agg(row_to_json(row)), '[]'::JSON)
        FROM (
            SELECT
                id,
                access,
                username
            FROM
                users
            WHERE
                username ~ $1
            ORDER BY
                username
            LIMIT $2::SmallInt
        ) row;
    ", &[ &filter, &limit ]) {
        Ok(rows) => Ok(rows.get(0).get(0)),
        Err(err) => Err(HecateError::from_db(err))
    }
}
