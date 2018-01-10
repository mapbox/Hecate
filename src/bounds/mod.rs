extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;

#[derive(PartialEq, Debug)]
pub enum BoundsError {
    NotFound,
    ListError(String),
    GetError(String)
}

impl UserError {
    pub fn to_string(&self) -> String {
        match *self {
            UserError::NotFound => String::from("User Not Found"),
            UserError::ListError(ref msg) => String::from(format!("Could not list bounds: {}", msg)),
            UserError::GetError(ref msg) => String::from(format!("Could not get bounds: {}", msg))
        }
    }
}

pub fn list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<bool, UserError> {
    match conn.query("
        SELECT name FROM 
    ", &[ &username, &password, &email ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(UserError::CreateError(e.message.clone())) },
                _ => Err(UserError::CreateError(String::from("generic")))
            }
        }
    }
}

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bounds: String) -> Result<Option<i64>, UserError> {
    match conn.query("
        SELECT
            id
        FROM
            users
        WHERE
            username = $1
            AND password = crypt($2, password)
    ", &[ &username, &password ]) {
        Ok(res) => {
            if res.len() == 0 { return Ok(None); }
            let uid: i64 = res.get(0).get(0);

            Ok(Some(uid))
        },
        Err(_) => Err(UserError::NotFound)
    }
}
