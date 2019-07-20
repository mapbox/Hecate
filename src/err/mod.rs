#[derive(PartialEq, Debug)]
pub struct HecateError {
    code: u16,
    custom_json: Option<serde_json::Value>,
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
            custom_json: None,
            safe_error: safe_error,
            full_error: full_error
        }
    }

    pub fn from_json(code: u16, json: serde_json::Value, safe_error: String, full_error: Option<String>) -> Self {
        let full_error = match full_error {
            Some(err) => err,
            None => safe_error.clone()
        };

        HecateError {
            code: code,
            custom_json: Some(json),
            safe_error: safe_error,
            full_error: full_error
        }
    }

    pub fn from_db(error: postgres::error::Error) -> Self {
        println!("Database Error: {:?}", &error);

        match error.as_db() {
            Some(db_err) => HecateError {
                code: 500,
                custom_json: None,
                safe_error: String::from("Database Error"),
                full_error: format!("{}", db_err)
            },
            None => HecateError {
                code: 500,
                custom_json: None,
                safe_error: String::from("Database Error"),
                full_error: format!("{}", error)
            }
        }
    }

    pub fn as_json(self) -> serde_json::Value {
        match self.custom_json {
            Some(custom_json) => custom_json,
            None => {
                let status = actix_web::http::StatusCode::from_u16(self.code).unwrap();

                json!({
                    "code": self.code,
                    "status": status.canonical_reason(),
                    "reason": self.safe_error
                })
            }
        }
    }
}

impl actix_web::Responder for HecateError {
    type Error = actix_web::Error;
    type Future = Result<actix_http::Response, actix_web::Error>;

    fn respond_to(self, req: &actix_web::HttpRequest) -> Self::Future {
        println!("HecateError: {:?} {} {}", &self.code, &self.safe_error, &self.full_error);

        let code = self.code;

        let body = match serde_json::to_string(&self.as_json()) {
            Ok(body) => body,
            Err(e) => return Err(e.into()),
        };

        Ok(actix_http::Response::build(actix_web::http::StatusCode::from_u16(code).unwrap())
           .content_type("application/json")
           .body(body))
    }
}

impl ToString for HecateError {
    fn to_string(&self) -> String {
        self.safe_error.clone()
    }
}
