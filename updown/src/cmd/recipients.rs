//! Subcommands for the `recipients` resource (`GET/POST/DELETE /api/recipients`).

use anyhow::Result;
use clap::Subcommand;
use std::collections::HashMap;

use crate::output::{self, OutputMode};
use updown_lib::client::Client;
use updown_lib::models::recipient::{Recipient, RecipientRow};

/// Actions available under `updown recipients`.
#[derive(Subcommand)]
pub enum RecipientsAction {
    /// List all recipients
    List,
    /// Create a new recipient
    Create {
        /// Recipient type
        #[arg(value_parser = ["email", "sms", "webhook", "slack_compatible", "msteams"])]
        recipient_type: String,
        /// Recipient value (email address, phone number, URL, etc.)
        value: String,
        #[arg(long)]
        name: Option<String>,
        /// Opt out of auto-selecting for all checks (API defaults to selected=true)
        #[arg(long)]
        no_selected: bool,
    },
    /// Delete a recipient
    Delete {
        /// Recipient ID
        id: String,
    },
}

/// Dispatches the requested recipients action to the appropriate API call.
pub fn run(action: RecipientsAction, client: &Client, mode: OutputMode) -> Result<()> {
    match action {
        RecipientsAction::List => list(client, mode),
        RecipientsAction::Create {
            recipient_type,
            value,
            name,
            no_selected,
        } => create(
            client,
            mode,
            &recipient_type,
            &value,
            name.as_deref(),
            no_selected,
        ),
        RecipientsAction::Delete { id } => delete(client, mode, &id),
    }
}

fn list(client: &Client, mode: OutputMode) -> Result<()> {
    let resp = client.get("/api/recipients")?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let recipients: Vec<Recipient> = resp.json()?;
            let rows: Vec<RecipientRow> = recipients.iter().map(RecipientRow::from).collect();
            output::print_table(&rows);
        }
        OutputMode::Axi => {
            let recipients: Vec<Recipient> = resp.json()?;
            let mut type_counts: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for r in &recipients {
                *type_counts.entry(r.recipient_type.as_str()).or_insert(0) += 1;
            }
            let mut counts: Vec<_> = type_counts.into_iter().collect();
            counts.sort_by_key(|(t, _)| t.to_string());
            let type_summary = counts
                .iter()
                .map(|(t, c)| format!("{} {}", c, t))
                .collect::<Vec<_>>()
                .join(", ");
            let aggregate = format!("{} recipients ({})", recipients.len(), type_summary);
            let rows: Vec<RecipientRow> = recipients.iter().map(RecipientRow::from).collect();
            output::print_axi_list(
                &rows,
                "recipients",
                &aggregate,
                &["recipients create <type> <value>"],
            );
        }
    }

    Ok(())
}

fn create(
    client: &Client,
    mode: OutputMode,
    recipient_type: &str,
    value: &str,
    name: Option<&str>,
    no_selected: bool,
) -> Result<()> {
    let mut body = HashMap::new();
    body.insert(
        "type".to_string(),
        serde_json::Value::String(recipient_type.to_string()),
    );
    body.insert(
        "value".to_string(),
        serde_json::Value::String(value.to_string()),
    );
    if let Some(n) = name {
        body.insert("name".to_string(), serde_json::Value::String(n.to_string()));
    }
    if no_selected {
        body.insert("selected".to_string(), serde_json::json!(false));
    }

    let resp = client.post("/api/recipients", &body)?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let r: Recipient = resp.json()?;
            output::print_confirm(&format!(
                "Recipient created: {} ({} {})",
                r.id,
                r.recipient_type,
                r.value.unwrap_or_default()
            ));
        }
        OutputMode::Axi => {
            let r: Recipient = resp.json()?;
            output::print_axi_confirm(
                &format!(
                    "Recipient created: {} ({} {})",
                    r.id,
                    r.recipient_type,
                    r.value.unwrap_or_default()
                ),
                &["recipients list"],
            );
        }
    }

    Ok(())
}

fn delete(client: &Client, mode: OutputMode, id: &str) -> Result<()> {
    let resp = client.delete(&format!("/api/recipients/{}", id))?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            output::print_confirm(&format!("Deleted recipient {}", id));
        }
        OutputMode::Axi => {
            let _json: serde_json::Value = resp.json()?;
            output::print_axi_confirm(
                &format!("Deleted recipient {}", id),
                &["recipients list"],
            );
        }
    }

    Ok(())
}
