//! Status page model — public-facing uptime dashboards in updown.io.

use serde::{Deserialize, Serialize};
use tabled::Tabled;

/// A status page as returned by the updown.io API.
///
/// Status pages aggregate one or more checks into a publicly shareable
/// uptime dashboard. Visibility can be `"public"`, `"protected"` (access key
/// required), or `"private"`.
#[derive(Debug, Deserialize, Serialize)]
pub struct StatusPage {
    /// Unique status page identifier used in all API paths.
    pub token: String,
    /// Public URL of the rendered status page.
    #[serde(default)]
    pub url: Option<String>,
    /// Display name shown on the status page.
    #[serde(default)]
    pub name: Option<String>,
    /// Optional description rendered below the page title.
    #[serde(default)]
    pub description: Option<String>,
    /// Access level: `"public"`, `"protected"`, or `"private"`.
    #[serde(default)]
    pub visibility: Option<String>,
    /// Tokens of the checks displayed on this page.
    #[serde(default)]
    pub checks: Option<Vec<String>>,
}

/// Flattened, display-ready representation of a [`StatusPage`] for table output.
#[derive(Debug, Tabled)]
pub struct StatusPageRow {
    #[tabled(rename = "TOKEN")]
    pub token: String,
    #[tabled(rename = "NAME")]
    pub name: String,
    #[tabled(rename = "VISIBILITY")]
    pub visibility: String,
    #[tabled(rename = "URL")]
    pub url: String,
    /// Comma-separated list of check tokens.
    #[tabled(rename = "CHECKS")]
    pub checks: String,
}

impl From<&StatusPage> for StatusPageRow {
    fn from(sp: &StatusPage) -> Self {
        StatusPageRow {
            token: sp.token.clone(),
            name: sp.name.clone().unwrap_or("-".to_string()),
            visibility: sp.visibility.clone().unwrap_or("-".to_string()),
            url: sp.url.clone().unwrap_or("-".to_string()),
            checks: sp
                .checks
                .as_ref()
                .map(|c| c.join(", "))
                .unwrap_or("-".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_status_page() {
        let json = r#"{
            "token": "sp123",
            "url": "https://status.example.com",
            "name": "My Status Page",
            "description": "System status",
            "visibility": "public",
            "checks": ["check1", "check2"]
        }"#;

        let sp: StatusPage = serde_json::from_str(json).unwrap();
        assert_eq!(sp.token, "sp123");
        assert_eq!(
            sp.checks,
            Some(vec!["check1".to_string(), "check2".to_string()])
        );
    }
}
