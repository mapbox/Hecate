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

    pub fn as_string(&self) -> String {
        String::new()
    }

    pub fn code_as_string(code: &i16) -> String {
        match *code {
            // 100 Status Codes (Informational)
            100 => String::from("Continue"),
            101 => String::from("Switching Protocols"),
            102 => String::from("Processing"),

            // 200 Status Codes (Success)
            200 => String::from("OK"),
            201 => String::from("Created"),
            202 => String::from("Accepted"),
            203 => String::from("Non-Authoritative Information"),
            204 => String::from("No Content"),
            205 => String::from("Reset Content"),
            206 => String::from("Partial Content"),

            // 300 Status Codes (Redirection)
            300 => String::from("Multiple Choices"),
            301 => String::from("Moved Permanently"),
            302 => String::from("Found"),
            303 => String::from("See Other"),
            304 => String::from("Not Modified"),
            305 => String::from("Use Proxy"),
            307 => String::from("Temporary Redirect"),
            308 => String::from("Permanent Redirect"),

            // 400 Status Codes (Client Error)
            400 => String::from("Bad Request"),
            401 => String::from("Unauthorized"),
            402 => String::from("Payment Required"),
            403 => String::from("Forbidden"),
            404 => String::from("Not Found"),
            405 => String::from("Method Not Allowed"),
            406 => String::from("Not Acceptable"),
            407 => String::from("Proxy Authentication Required"),
            408 => String::from("Request Timeout"),
            409 => String::from("Conflict"),
            410 => String::from("Gone"),
            411 => String::from("Length Required"),
            412 => String::from("Precondition Failed"),
            413 => String::from(""),
            414 => String::from(""),
            415 => String::from(""),
            416 => String::from(""),
            417 => String::from(""),
            418 => String::from(""),
            426 => String::from(""),
            428 => String::from(""),
            429 => String::from(""),
            431 => String::from(""),
            451 => String::from(""),

            // 500 Status Codes (Server Error)
            500 => String::from(""),
            501 => String::from(""),
            502 => String::from(""),
            503 => String::from(""),
            504 => String::from(""),
            505 => String::from("")
        }
    }

    pub fn as_json(self) -> serde_json::Value {
        match self.custom_json {
            Some(custom_json) => custom_json,
            None => json!({
                "code": self.code,
                "status": code_as_string(&self.code),
                "reason": self.safe_error
            })
        }
    }
}

use std::io::Cursor;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;

impl <'r> Responder<'r> for HecateError {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        // TODO ADD STATUS CODE
        Response::build()
            .sized_body(Cursor::new(self.as_json().to_string()))
            .header(ContentType::new("application", "json"))
            .ok()
    }
}
