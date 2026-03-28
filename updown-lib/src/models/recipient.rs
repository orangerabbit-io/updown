//! Recipient model — alert notification targets in updown.io.

use serde::{Deserialize, Serialize};
use tabled::Tabled;

/// An alert recipient as returned by the updown.io API.
///
/// Recipients receive notifications when a check goes down or recovers.
/// Supported types include `email`, `sms`, `webhook`, `slack_compatible`,
/// and `msteams`.
#[derive(Debug, Deserialize, Serialize)]
pub struct Recipient {
    /// Unique recipient identifier.
    pub id: String,
    /// Notification channel type (e.g. `"email"`, `"webhook"`).
    #[serde(rename = "type")]
    pub recipient_type: String,
    /// Optional display name for the recipient.
    #[serde(default)]
    pub name: Option<String>,
    /// The notification target: email address, phone number, or URL depending on type.
    #[serde(default)]
    pub value: Option<String>,
    /// Whether this recipient is auto-selected when creating new checks.
    #[serde(default)]
    pub selected: Option<bool>,
}

/// Flattened, display-ready representation of a [`Recipient`] for table output.
#[derive(Debug, Serialize, Tabled)]
pub struct RecipientRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "TYPE")]
    pub recipient_type: String,
    #[tabled(rename = "NAME")]
    pub name: String,
    #[tabled(rename = "VALUE")]
    pub value: String,
    #[tabled(rename = "SELECTED")]
    pub selected: String,
}

impl From<&Recipient> for RecipientRow {
    fn from(r: &Recipient) -> Self {
        RecipientRow {
            id: r.id.clone(),
            recipient_type: r.recipient_type.clone(),
            name: r.name.clone().unwrap_or("-".to_string()),
            value: r.value.clone().unwrap_or("-".to_string()),
            selected: r.selected.map(|s| s.to_string()).unwrap_or("-".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_recipient() {
        let json = r#"{
            "id": "rec123",
            "type": "email",
            "name": "Admin",
            "value": "admin@example.com",
            "selected": true
        }"#;

        let r: Recipient = serde_json::from_str(json).unwrap();
        assert_eq!(r.id, "rec123");
        assert_eq!(r.recipient_type, "email");
        assert_eq!(r.selected, Some(true));
    }
}
