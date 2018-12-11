#[derive(PartialEq, Debug)]
pub struct HecateError {
    code: u16,
    safe_error: String,
    full_error: String
}

impl HecateError {
    pub fn new(code: u16, safe_error: String, full_error: Option<String>) -> Self {
        let full_error = match full_error {
            Some(err) => err,
            None => safe_error.clone()
        };

        HecateError {
            code: code,
            safe_error: safe_error,
            full_error: full_error
        }
    }

    pub fn from_db(error: postgres::error::Error) -> Self {
        match error.as_db() {
            Some(db_err) => HecateError {
                code: 500,
                safe_error: String::from("Database Error"),
                full_error: format!("{}", db_err)
            },
            None => HecateError {
                code: 500,
                safe_error: String::from("Database Error"),
                full_error: format!("{}", error)
            }
        }
    }

    pub fn as_string(&self) -> String {
        String::new()
    }

    pub fn as_json(&self) -> serde_json::Value {
        json!({
            "code": self.code,
            "status": "Service Unavailable", //TODO: Lookup Status from code
            "reason": self.safe_error
        })
    }

    pub fn as_resp(&self) -> rocket::response::status::Custom<rocket_contrib::json::Json<serde_json::Value>> {
                                            //TODO: Obv this needs to be selected based on code
        rocket::response::status::Custom(rocket::http::Status::ServiceUnavailable, rocket_contrib::json::Json(self.as_json()))
    }
}
