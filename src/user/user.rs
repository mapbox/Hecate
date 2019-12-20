use crate::err::HecateError;
use crate::validate;

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
            access: Some(String::from("default"))
        }
    }

    pub fn to_value(self) -> serde_json::Value {
        let access = match self.access {
            Some(access) => access,
            None => String::from("default")
        };

        json!({
            "id": self.id,
            "username": self.username,
            "email": self.email,
            "meta": self.meta,
            "access": access
        })
    }

    pub fn reset(conn: &impl postgres::GenericConnection, uid: &i64, current: &String, new: &String) -> Result<bool, HecateError> {
        validate::password(&new)?;

        match conn.query("
            UPDATE users
                SET
                    password = crypt($3, gen_salt('bf', 10))
                WHERE
                    id = $1
                    AND password = crypt($2, password)
                RETURNING
                    users
        ", &[ &uid, &current, &new ]) {
            Ok(users) => {
                if users.len() == 0 {
                    Err(HecateError::new(400, String::from("Incorrect Current Password"), None))
                } else {
                    Ok(true)
                }
            },
            Err(err) => Err(HecateError::from_db(err))
        }
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
            false => self.access = Some(String::from("default"))
        };
    }

    pub fn set(&self, conn: &impl postgres::GenericConnection) -> Result<bool, HecateError> {
        if self.id.is_some() {
            match conn.query("
                UPDATE users
                    SET
                        username = COALESCE($1, username),
                        email = COALESCE($2, email),
                        meta = COALESCE($3, meta),
                        access = COALESCE($4, access)
                    WHERE
                        id = $5
            ", &[ &self.username, &self.email, &self.meta, &self.access, &self.id ]) {
                Ok(_) => (),
                Err(err) => { return Err(HecateError::from_db(err)); }
            };

            if self.access != Some(String::from("disabled")) {
                return Ok(true);
            }

            // Once the user is disable, cycle the password to prevent
            // accidental enablement
            match conn.query("
                UPDATE users
                    SET
                        password = crypt(random()::TEXT, gen_salt('bf', 10))
                    WHERE
                        id = $1
            ", &[ &self.id ]) {
                Ok(_) => (),
                Err(err) => { return Err(HecateError::from_db(err)); }
            };

            // Destroy all tokens associated with a disabled user
            match conn.query("
                DELETE FROM users_tokens
                    WHERE
                        uid = $1
            ", &[ &self.id ]) {
                Ok(_) => (),
                Err(err) => { return Err(HecateError::from_db(err)); }
            };

            Ok(true)
        } else {
            let password = match self.password {
                Some(ref password) => {
                    validate::password(password)?;

                    password
                },
                None => { return Err(HecateError::new(400, String::from("Password must be present to create new user"), None)); }
            };

            match conn.query("
                INSERT INTO users (username, password, email, meta, access)
                    VALUES ($1, crypt($2, gen_salt('bf', 10)), $3, $4, $5);
            ", &[ &self.username, &password, &self.email, &self.meta, &self.access.as_ref().unwrap_or(&String::from("default")) ]) {
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
