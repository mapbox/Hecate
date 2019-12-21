use postgres;
use reqwest;
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use url::Url;
use base64;

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

use crate::{
    worker,
    err::HecateError
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WebHook {
    pub id: Option<i64>,
    pub name: String,
    pub actions: Vec<String>,
    pub url: String,
    pub secret: Option<String>
}

impl WebHook {
    pub fn new(id: i64, name: String, actions: Vec<String>, url: String, secret: Option<String>) -> Self {
        WebHook {
            id: Some(id),
            name,
            actions,
            url,
            secret
        }
    }

    pub fn to_value(self) -> serde_json::Value {
        json!({
            "id": self.id,
            "name": self.name,
            "actions": self.actions,
            "url": self.url
        })
    }
}

#[derive(Debug)]
pub enum Action {
    All,
    User,
    Delta,
    Meta,
    Style
}

pub fn list(conn: &impl postgres::GenericConnection, action: Action) -> Result<Vec<WebHook>, HecateError> {
    let action = match action {
        Action::All => "",
        Action::User => "WHERE actions @>ARRAY['user']",
        Action::Delta => "WHERE actions @>ARRAY['delta']",
        Action::Meta => "WHERE actions @>ARRAY['meta']",
        Action::Style => "WHERE actions @>ARRAY['style']"
    };

    match conn.query(format!("
        SELECT
            id,
            name,
            actions,
            url,
            secret
        FROM
            webhooks
        {action}
    ", action = &action).as_str(), &[]) {
        Ok(results) => {
            let mut hooks: Vec<WebHook> = Vec::with_capacity(results.len());

            for result in results.iter() {
                hooks.push(WebHook::new(result.get(0), result.get(1), result.get(2), result.get(3), result.get(4)));
            }

            Ok(hooks)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn get(conn: &impl postgres::GenericConnection, id: i64) -> Result<WebHook, HecateError> {
    match conn.query("
        SELECT
            id,
            name,
            actions,
            url,
            secret
        FROM
            webhooks
        WHERE
            id = $1
    ", &[&id]) {
        Ok(results) => {
            if results.len() == 0 {
                return Err(HecateError::new(404, String::from("Webhook Not Found"), None));
            }

            let result = results.get(0);
            Ok(WebHook::new(result.get(0), result.get(1), result.get(2), result.get(3), result.get(4)))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn delete(conn: &impl postgres::GenericConnection, id: i64) -> Result<bool, HecateError> {
    match conn.execute("
        DELETE FROM webhooks
        WHERE id = $1
    ", &[&id]) {
        Ok(_) => Ok(true),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn create(conn: &impl postgres::GenericConnection, mut webhook: WebHook) -> Result<WebHook, HecateError> {
    if !is_valid_action(&webhook.actions) {
        return Err(HecateError::new(400, String::from("Invalid Action"), None));
    }

    if Url::parse(&webhook.url).is_err() {
        return Err(HecateError::new(422, String::from("Invalid webhook url"), None))
    }

    webhook.secret = match webhook.secret {
        Some(secret) => Some(secret),
        None => {
            // if no secret exists, generate a 30 char alphanumeric secret
            let secret: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .collect();
            Some(secret)
        }
    };

    match conn.query("
        INSERT INTO webhooks (name, actions, url, secret)
            VALUES (
                $1,
                $2,
                $3,
                $4
            )
            Returning id
    ", &[&webhook.name, &webhook.actions, &webhook.url, &webhook.secret]) {
        Ok(results) => {
            let id = results.get(0).get(0);

            webhook.id = Some(id);

            Ok(webhook.clone())
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn update(conn: &impl postgres::GenericConnection, webhook: WebHook) -> Result<WebHook, HecateError> {
    if !is_valid_action(&webhook.actions) {
        return Err(HecateError::new(400, String::from("Invalid Action"), None));
    }

    if Url::parse(&webhook.url).is_err() {
        return Err(HecateError::new(422, String::from("Invalid webhook url"), None))
    }

    match conn.execute("
         UPDATE webhooks
            SET
                name = $1,
                actions = $2,
                url = $3
            WHERE id = $4
    ", &[&webhook.name, &webhook.actions, &webhook.url, &webhook.id]) {
        Ok(_) => Ok(webhook),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn is_valid_action(actions: &Vec<String>) -> bool {
    for action in actions {
        if
            action != "delta"
            && action != "user"
            && action != "meta"
            && action != "style"
        {
            return false;
        }
    }

    true
}

pub fn send(conn: &impl postgres::GenericConnection, task: &worker::TaskType) -> Result<(), HecateError> {
    let action = match task {
        worker::TaskType::Delta(_) => Action::Delta,
        worker::TaskType::User(_) => Action::User,
        worker::TaskType::Style(_) => Action::Style,
        worker::TaskType::Meta => Action::Meta
    };

    for hook in list(conn, action)? {
        let client = reqwest::Client::new();

        let body = match task {
            worker::TaskType::Delta(delta) => {
                json!({
                    "id": delta,
                    "type": "delta"
                }).to_string()
            },
            worker::TaskType::User(user) => {
                json!({
                    "id": user,
                    "type": "user"
                }).to_string()
            },
            worker::TaskType::Style(style) => {
                json!({
                    "id": style,
                    "type": "style"
                }).to_string()
            },
            worker::TaskType::Meta => {
                json!({
                    "id": null,
                    "type": "meta"
                }).to_string()
            }
        };

        let mut mac = match HmacSha256::new_varkey(hook.secret.unwrap().as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return Err(HecateError::new(500, String::from("Internal Server Error"), None))
        };
        mac.input(body.as_bytes());
        let signature = base64::encode(&mac.result().code());

        let url = match Url::parse_with_params(hook.url.as_str(), &[("signature", signature)]) {
            Ok(url) => url,
            Err(_) => return Err(HecateError::new(500, String::from("Internal Server Error"), None))
        };

        match client.post(url.as_str())
            .body(body)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
        {
            Ok(res) => {
                println!("Successfully sent webhook {}: {:#?}", hook.url, res);
            },
            Err(err) => {
                println!("WARN: Failed to post to webhook {}: {:?}", hook.url, err);
            }
        };
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn webhooks_new() {
        assert_eq!(
            WebHook::new(
                1,
                String::from("webhook"),
                vec![String::from("delta")],
                String::from("www.example.com"),
                None),
            WebHook {
                id: Some(1),
                name: String::from("webhook"),
                actions: vec![String::from("delta")],
                url: String::from("www.example.com"),
                secret: None
            }
        );
    }
}
