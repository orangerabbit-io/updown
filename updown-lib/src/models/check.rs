//! Check model — the primary monitoring resource in updown.io.

use serde::{Deserialize, Serialize};
use tabled::Tabled;

/// A monitoring check as returned by the updown.io API.
///
/// Checks poll a URL (or host, for ICMP/TCP) at a configurable interval and
/// record uptime, response codes, and Apdex scores. Most fields are optional
/// because the API omits them when they are not applicable.
#[derive(Debug, Deserialize, Serialize)]
pub struct Check {
    /// Unique check identifier used in all API paths.
    pub token: String,
    /// The URL (or host) being monitored.
    pub url: String,
    /// Optional human-readable label for the check.
    #[serde(default)]
    pub alias: Option<String>,
    /// HTTP status code from the most recent probe.
    #[serde(default)]
    pub last_status: Option<u16>,
    /// Rolling uptime percentage (0–100).
    #[serde(default)]
    pub uptime: Option<f64>,
    /// `true` when the check is currently reporting a failure.
    #[serde(default)]
    pub down: Option<bool>,
    /// Polling interval in seconds.
    #[serde(default)]
    pub period: Option<u32>,
    /// Apdex satisfaction threshold in seconds.
    #[serde(default)]
    pub apdex_t: Option<f64>,
    /// Whether the check is actively polling.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Whether the check appears on public status pages.
    #[serde(default)]
    pub published: Option<bool>,
    /// Substring that must appear in the response body for the check to pass.
    #[serde(default)]
    pub string_match: Option<String>,
    /// ISO 8601 timestamp or `"recovery"` — alerts are suppressed until this time.
    #[serde(default)]
    pub mute_until: Option<String>,
    /// HTTP verb used for the request (e.g. `"GET/HEAD"`, `"POST"`).
    #[serde(default)]
    pub http_verb: Option<String>,
    /// Request body sent with POST/PUT probes.
    #[serde(default)]
    pub http_body: Option<String>,
    /// Check type: `https`, `http`, `icmp`, `pulse`, `tcp`, or `tcps`.
    #[serde(default, rename = "type")]
    pub check_type: Option<String>,
    /// Node location codes excluded from probing (e.g. `["fra", "sin"]`).
    #[serde(default)]
    pub disabled_locations: Option<Vec<String>>,
    /// Alert recipients attached to this check.
    #[serde(default)]
    pub recipients: Option<Vec<serde_json::Value>>,
    /// Custom HTTP headers sent with each probe, as a JSON object.
    #[serde(default)]
    pub custom_headers: Option<serde_json::Value>,
    /// Aggregated metrics, present only when requested via `?metrics=true`.
    #[serde(default)]
    pub metrics: Option<CheckMetrics>,
}

/// Flattened, display-ready representation of a [`Check`] for table output.
#[derive(Debug, Tabled)]
pub struct CheckRow {
    #[tabled(rename = "TOKEN")]
    pub token: String,
    /// `"up"` or `"down"` derived from [`Check::down`].
    #[tabled(rename = "STATUS")]
    pub status: String,
    #[tabled(rename = "URL")]
    pub url: String,
    /// Formatted as `"99.97%"`, or `"-"` when not yet available.
    #[tabled(rename = "UPTIME")]
    pub uptime: String,
    /// Apdex threshold formatted to two decimal places, or `"-"`.
    #[tabled(rename = "APDEX")]
    pub apdex: String,
    /// Polling interval formatted as `"60s"`, or `"-"`.
    #[tabled(rename = "PERIOD")]
    pub period: String,
}

impl From<&Check> for CheckRow {
    fn from(c: &Check) -> Self {
        CheckRow {
            token: c.token.clone(),
            status: if c.down == Some(true) {
                "down".to_string()
            } else {
                "up".to_string()
            },
            url: c.url.clone(),
            uptime: c
                .uptime
                .map(|u| format!("{:.2}%", u))
                .unwrap_or("-".to_string()),
            apdex: c
                .apdex_t
                .map(|a| format!("{:.2}", a))
                .unwrap_or("-".to_string()),
            period: c
                .period
                .map(|p| format!("{}s", p))
                .unwrap_or("-".to_string()),
        }
    }
}

/// Aggregated performance metrics for a check, returned when `?metrics=true` is set.
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckMetrics {
    /// Overall Apdex score for the requested period.
    #[serde(default)]
    pub apdex: Option<f64>,
    /// Breakdown of response time components (DNS, connect, TLS, etc.).
    #[serde(default)]
    pub timings: Option<serde_json::Value>,
    /// Request counts and error rates for the requested period.
    #[serde(default)]
    pub requests: Option<serde_json::Value>,
}

/// A single downtime event recorded by updown.io for a check.
#[derive(Debug, Deserialize, Serialize)]
pub struct Downtime {
    /// Unique identifier for this downtime event.
    pub id: String,
    /// Human-readable description of the failure (e.g. `"Connection refused"`).
    #[serde(default)]
    pub error: Option<String>,
    /// ISO 8601 timestamp when the downtime began.
    #[serde(default)]
    pub started_at: Option<String>,
    /// ISO 8601 timestamp when the check recovered. `None` if still down.
    #[serde(default)]
    pub ended_at: Option<String>,
    /// Duration of the outage in seconds.
    #[serde(default)]
    pub duration: Option<u64>,
}

/// Flattened, display-ready representation of a [`Downtime`] for table output.
#[derive(Debug, Tabled)]
pub struct DowntimeRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "ERROR")]
    pub error: String,
    #[tabled(rename = "STARTED")]
    pub started_at: String,
    #[tabled(rename = "ENDED")]
    pub ended_at: String,
    /// Duration formatted as `"3600s"`, or `"-"` when still ongoing.
    #[tabled(rename = "DURATION")]
    pub duration: String,
}

impl From<&Downtime> for DowntimeRow {
    fn from(d: &Downtime) -> Self {
        DowntimeRow {
            id: d.id.clone(),
            error: d.error.clone().unwrap_or("-".to_string()),
            started_at: d.started_at.clone().unwrap_or("-".to_string()),
            ended_at: d.ended_at.clone().unwrap_or("-".to_string()),
            duration: d
                .duration
                .map(|s| format!("{}s", s))
                .unwrap_or("-".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_check() {
        let json = r#"{
            "token": "abc123",
            "url": "https://example.com",
            "alias": "My Site",
            "last_status": 200,
            "uptime": 99.97,
            "down": false,
            "period": 60,
            "apdex_t": 0.5,
            "enabled": true,
            "published": false,
            "type": "https"
        }"#;

        let check: Check = serde_json::from_str(json).unwrap();
        assert_eq!(check.token, "abc123");
        assert_eq!(check.url, "https://example.com");
        assert_eq!(check.alias, Some("My Site".to_string()));
        assert_eq!(check.down, Some(false));
        assert_eq!(check.period, Some(60));
        assert_eq!(check.check_type, Some("https".to_string()));
    }

    #[test]
    fn test_check_row_up() {
        let check = Check {
            token: "abc".to_string(),
            url: "https://example.com".to_string(),
            alias: None,
            last_status: Some(200),
            uptime: Some(99.97),
            down: Some(false),
            period: Some(30),
            apdex_t: Some(0.98),
            enabled: None,
            published: None,
            string_match: None,
            mute_until: None,
            http_verb: None,
            http_body: None,
            check_type: None,
            disabled_locations: None,
            recipients: None,
            custom_headers: None,
            metrics: None,
        };

        let row = CheckRow::from(&check);
        assert_eq!(row.status, "up");
        assert_eq!(row.uptime, "99.97%");
        assert_eq!(row.period, "30s");
    }

    #[test]
    fn test_deserialize_downtime() {
        let json = r#"{
            "id": "dt123",
            "error": "Connection refused",
            "started_at": "2024-01-01T00:00:00Z",
            "ended_at": "2024-01-01T01:00:00Z",
            "duration": 3600
        }"#;

        let dt: Downtime = serde_json::from_str(json).unwrap();
        assert_eq!(dt.id, "dt123");
        assert_eq!(dt.duration, Some(3600));
    }
}
