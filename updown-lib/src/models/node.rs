//! Node model — updown.io monitoring probe locations.

use serde::{Deserialize, Serialize};
use tabled::Tabled;

/// A monitoring node (probe location) as returned by `GET /api/nodes`.
///
/// The API response is a map from location code (e.g. `"fra"`) to `Node`.
/// The code is not embedded in the struct itself.
#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
    /// IPv4 address of the probe.
    #[serde(default)]
    pub ip: Option<String>,
    /// IPv6 address of the probe.
    #[serde(default)]
    pub ip6: Option<String>,
    /// City where the probe is hosted.
    #[serde(default)]
    pub city: Option<String>,
    /// Full country name.
    #[serde(default)]
    pub country: Option<String>,
    /// ISO 3166-1 alpha-2 country code.
    #[serde(default)]
    pub country_code: Option<String>,
    /// Latitude of the probe location.
    #[serde(default)]
    pub lat: Option<f64>,
    /// Longitude of the probe location.
    #[serde(default)]
    pub lng: Option<f64>,
}

/// Flattened, display-ready representation of a node for table output.
///
/// Includes the location `code` key from the API response map, which is
/// not part of the [`Node`] struct itself.
#[derive(Debug, Serialize, Tabled)]
pub struct NodeRow {
    /// Location code (e.g. `"fra"`, `"sin"`).
    #[tabled(rename = "CODE")]
    pub code: String,
    #[tabled(rename = "CITY")]
    pub city: String,
    #[tabled(rename = "COUNTRY")]
    pub country: String,
    #[tabled(rename = "IPV4")]
    pub ip: String,
    #[tabled(rename = "IPV6")]
    pub ip6: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_node() {
        let json = r#"{
            "ip": "1.2.3.4",
            "ip6": "::1",
            "city": "Paris",
            "country": "France",
            "country_code": "FR",
            "lat": 48.8566,
            "lng": 2.3522
        }"#;

        let node: Node = serde_json::from_str(json).unwrap();
        assert_eq!(node.city, Some("Paris".to_string()));
        assert_eq!(node.ip, Some("1.2.3.4".to_string()));
    }
}
