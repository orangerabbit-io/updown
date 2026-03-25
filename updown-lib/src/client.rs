//! Authenticated HTTP client for the updown.io REST API.
//!
//! All requests attach the `X-API-KEY` header and request gzip encoding.
//! Non-2xx responses are converted into typed errors before being returned.

use anyhow::{bail, Context, Result};
use reqwest::blocking::{Client as HttpClient, Response};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_ENCODING};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Blocking HTTP client preconfigured for the updown.io API.
///
/// Holds the base URL and API key so callers never need to supply auth headers
/// directly. Use [`Client::new`] to construct.
pub struct Client {
    http: HttpClient,
    base_url: String,
    api_key: String,
}

impl Client {
    /// Constructs a new client with gzip encoding enabled.
    ///
    /// `base_url` should be `https://updown.io` in production. The `UPDOWN_BASE_URL`
    /// environment variable can override this for local development or testing.
    pub fn new(api_key: String, base_url: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));

        let http = HttpClient::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Client {
            http,
            base_url,
            api_key,
        })
    }

    /// Sends an authenticated GET request and returns the raw response.
    pub fn get(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .with_context(|| format!("Request failed: GET {}", url))?;
        Self::check_status(resp)
    }

    /// Sends an authenticated GET request with query parameters and returns the raw response.
    pub fn get_with_params(&self, path: &str, params: &[(&str, &str)]) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .query(params)
            .send()
            .with_context(|| format!("Request failed: GET {}", url))?;
        Self::check_status(resp)
    }

    /// Sends an authenticated POST request with a JSON body and returns the raw response.
    pub fn post(&self, path: &str, body: &HashMap<String, serde_json::Value>) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("X-API-KEY", &self.api_key)
            .json(body)
            .send()
            .with_context(|| format!("Request failed: POST {}", url))?;
        Self::check_status(resp)
    }

    /// Sends an authenticated PUT request with a JSON body and returns the raw response.
    pub fn put(&self, path: &str, body: &HashMap<String, serde_json::Value>) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .put(&url)
            .header("X-API-KEY", &self.api_key)
            .json(body)
            .send()
            .with_context(|| format!("Request failed: PUT {}", url))?;
        Self::check_status(resp)
    }

    /// Sends an authenticated DELETE request and returns the raw response.
    pub fn delete(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .delete(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .with_context(|| format!("Request failed: DELETE {}", url))?;
        Self::check_status(resp)
    }

    /// Sends a GET request and deserializes the JSON response body into `T`.
    #[allow(dead_code)]
    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self.get(path)?;
        resp.json::<T>().context("Failed to parse JSON response")
    }

    /// Sends a GET request with query parameters and deserializes the JSON response body into `T`.
    #[allow(dead_code)]
    pub fn get_json_with_params<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let resp = self.get_with_params(path, params)?;
        resp.json::<T>().context("Failed to parse JSON response")
    }

    /// Sends a GET request and returns the response body as a plain string.
    #[allow(dead_code)]
    pub fn get_text(&self, path: &str) -> Result<String> {
        let resp = self.get(path)?;
        resp.text().context("Failed to read response body")
    }

    /// Sends a GET request with query parameters and returns the response body as a plain string.
    ///
    /// Used for the nodes IP endpoints when `--format txt` is requested, which returns
    /// newline-delimited IP addresses rather than JSON.
    pub fn get_text_with_params(&self, path: &str, params: &[(&str, &str)]) -> Result<String> {
        let resp = self.get_with_params(path, params)?;
        resp.text().context("Failed to read response body")
    }

    fn check_status(resp: Response) -> Result<Response> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        let url = resp.url().to_string();
        let body = resp.text().unwrap_or_default();
        match status.as_u16() {
            401 | 403 => bail!("Authentication failed (HTTP {}): {}", status, body),
            404 => bail!("Not found (HTTP {}): {}", status, body),
            422 => bail!("Validation error (HTTP {}): {}", status, body),
            429 => bail!("Rate limited (HTTP {}): {}", status, body),
            _ => bail!("API error (HTTP {}) for {}: {}", status, url, body),
        }
    }
}
