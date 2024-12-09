// This will be a wrapper around reqwest::blocking::Client that provides a Trait we can use to for mocking in tests

use reqwest::blocking::Client as ReqwestClient;
use serde_json::Value as JsonValue;

// Trait for our http app service
#[cfg_attr(test, mockall::automock)]
pub trait HttpTrait {
    fn get(&self, token: &str, url: &str) -> Result<JsonValue, Box<dyn std::error::Error>>;
}

// A struct for our compute app service that implements the HttpTrait
pub struct Http {
    client: ReqwestClient,
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}

impl Http {
    // A builder function for our compute app service
    pub fn new() -> Self {
        Http {
            client: ReqwestClient::new(),
        }
    }
}

// Implement the HttpTrait for our Http struct
impl HttpTrait for Http {
    fn get(&self, token: &str, url: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
        let resp = self
            .client
            .get(url)
            .bearer_auth(token.to_owned())
            .send()?
            .json::<JsonValue>()?;
        Ok(resp)
    }
}
