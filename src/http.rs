//! This module provides an HTTP client abstraction and a concrete implementation using `reqwest`.
//! It also defines a trait `HttpTrait` for mocking in tests.

use reqwest::blocking::Client as ReqwestClient;
use serde_json::Value as JsonValue;

/// A trait defining the interface for an HTTP client.
/// This trait allows for mocking the HTTP client in tests.
#[cfg_attr(test, mockall::automock)]
pub trait HttpClient {
    /// Sends a GET request to the specified URL with the given bearer token.
    ///
    /// # Arguments
    ///
    /// * `token` - The bearer token for authentication.
    /// * `url` - The URL to send the request to.
    ///
    /// # Returns
    ///
    /// * `Ok(JsonValue)` - The JSON response from the server on success.
    /// * `Err(Box<dyn std::error::Error>)` - An error if the request fails.
    fn get(&self, token: &str, url: &str) -> Result<JsonValue, Box<dyn std::error::Error>>;
}

/// An HTTP client implementation using `reqwest`.
pub struct Http {
    /// The underlying `reqwest` client.
    client: ReqwestClient,
}

impl Default for Http {
    /// Creates a new `Http` client with the default `reqwest` configuration.
    fn default() -> Self {
        Self::new()
    }
}

impl Http {
    /// Creates a new `Http` client with the default `reqwest` configuration.
    pub fn new() -> Self {
        Http {
            client: ReqwestClient::new(),
        }
    }
}

// Implement the HttpTrait for our Http struct
impl HttpClient for Http {
    /// Sends a GET request using `reqwest`.
    ///
    /// This implementation uses the underlying `reqwest` client to send a GET request
    /// to the specified URL, including the provided bearer token for authentication.
    /// The response is parsed as JSON and returned.
    ///
    /// # Arguments
    ///
    /// * `token` - The bearer token for authentication.
    /// * `url` - The URL to send the request to.
    ///
    /// # Returns
    ///
    /// * `Ok(JsonValue)` - The JSON response from the server on success.
    /// * `Err(Box<dyn std::error::Error>)` - An error if the request fails,
    ///   including network errors, deserialization errors, and invalid token errors.
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
