//! Subcommands for the `status_pages` resource (`GET/POST/PUT/DELETE /api/status_pages`).

use anyhow::Result;
use clap::Subcommand;
use std::collections::HashMap;

use crate::output::{self, OutputMode};
use updown_lib::client::Client;
use updown_lib::models::status_page::{StatusPage, StatusPageRow};

/// Actions available under `updown status-pages`.
#[derive(Subcommand)]
pub enum StatusPagesAction {
    /// List all status pages
    List,
    /// Create a new status page
    Create {
        /// Check tokens to include (required)
        #[arg(long, value_delimiter = ',', required = true)]
        checks: Vec<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, value_parser = ["public", "protected", "private"])]
        visibility: Option<String>,
        #[arg(long)]
        access_key: Option<String>,
    },
    /// Update a status page
    Update {
        /// Status page token
        token: String,
        #[arg(long, value_delimiter = ',')]
        checks: Option<Vec<String>>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, value_parser = ["public", "protected", "private"])]
        visibility: Option<String>,
        #[arg(long)]
        access_key: Option<String>,
    },
    /// Delete a status page
    Delete {
        /// Status page token
        token: String,
    },
}

/// Dispatches the requested status pages action to the appropriate API call.
pub fn run(action: StatusPagesAction, client: &Client, mode: OutputMode) -> Result<()> {
    match action {
        StatusPagesAction::List => list(client, mode),
        StatusPagesAction::Create {
            checks,
            name,
            description,
            visibility,
            access_key,
        } => create(
            client,
            mode,
            checks,
            name,
            description,
            visibility,
            access_key,
        ),
        StatusPagesAction::Update {
            token,
            checks,
            name,
            description,
            visibility,
            access_key,
        } => update(
            client,
            mode,
            &token,
            checks,
            name,
            description,
            visibility,
            access_key,
        ),
        StatusPagesAction::Delete { token } => delete(client, mode, &token),
    }
}

fn list(client: &Client, mode: OutputMode) -> Result<()> {
    let resp = client.get("/api/status_pages")?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let pages: Vec<StatusPage> = resp.json()?;
            let rows: Vec<StatusPageRow> = pages.iter().map(StatusPageRow::from).collect();
            output::print_table(&rows);
        }
        OutputMode::Axi => {
            let pages: Vec<StatusPage> = resp.json()?;
            let public_count = pages
                .iter()
                .filter(|p| p.visibility.as_deref() == Some("public"))
                .count();
            let checks_count: usize = pages
                .iter()
                .map(|p| p.checks.as_ref().map(|c| c.len()).unwrap_or(0))
                .sum();
            let aggregate = format!(
                "{} status pages, {} public, {} total checks monitored",
                pages.len(),
                public_count,
                checks_count
            );
            let rows: Vec<StatusPageRow> = pages.iter().map(StatusPageRow::from).collect();
            output::print_axi_list(
                &rows,
                "status_pages",
                &aggregate,
                &["status-pages create --checks <tokens>"],
            );
        }
    }

    Ok(())
}

fn create(
    client: &Client,
    mode: OutputMode,
    checks: Vec<String>,
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
    access_key: Option<String>,
) -> Result<()> {
    let mut body = HashMap::new();
    body.insert("checks".to_string(), serde_json::json!(checks));
    if let Some(n) = &name {
        body.insert("name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(d) = &description {
        body.insert(
            "description".to_string(),
            serde_json::Value::String(d.clone()),
        );
    }
    if let Some(v) = &visibility {
        body.insert(
            "visibility".to_string(),
            serde_json::Value::String(v.clone()),
        );
    }
    if let Some(k) = &access_key {
        body.insert(
            "access_key".to_string(),
            serde_json::Value::String(k.clone()),
        );
    }

    let resp = client.post("/api/status_pages", &body)?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let sp: StatusPage = resp.json()?;
            output::print_confirm(&format!(
                "Status page created: {} ({})",
                sp.token,
                sp.url.unwrap_or_default()
            ));
        }
        OutputMode::Axi => {
            let sp: StatusPage = resp.json()?;
            output::print_axi_confirm(
                &format!(
                    "Status page created: {} ({})",
                    sp.token,
                    sp.url.unwrap_or_default()
                ),
                &["status-pages list"],
            );
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn update(
    client: &Client,
    mode: OutputMode,
    token: &str,
    checks: Option<Vec<String>>,
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
    access_key: Option<String>,
) -> Result<()> {
    let mut body = HashMap::new();
    if let Some(c) = &checks {
        body.insert("checks".to_string(), serde_json::json!(c));
    }
    if let Some(n) = &name {
        body.insert("name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(d) = &description {
        body.insert(
            "description".to_string(),
            serde_json::Value::String(d.clone()),
        );
    }
    if let Some(v) = &visibility {
        body.insert(
            "visibility".to_string(),
            serde_json::Value::String(v.clone()),
        );
    }
    if let Some(k) = &access_key {
        body.insert(
            "access_key".to_string(),
            serde_json::Value::String(k.clone()),
        );
    }

    let resp = client.put(&format!("/api/status_pages/{}", token), &body)?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let sp: StatusPage = resp.json()?;
            output::print_confirm(&format!("Status page updated: {}", sp.token));
        }
        OutputMode::Axi => {
            let sp: StatusPage = resp.json()?;
            output::print_axi_confirm(
                &format!("Status page updated: {}", sp.token),
                &["status-pages list"],
            );
        }
    }

    Ok(())
}

fn delete(client: &Client, mode: OutputMode, token: &str) -> Result<()> {
    let resp = client.delete(&format!("/api/status_pages/{}", token))?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            output::print_confirm(&format!("Deleted status page {}", token));
        }
        OutputMode::Axi => {
            output::print_axi_confirm(
                &format!("Deleted status page {}", token),
                &["status-pages list"],
            );
        }
    }

    Ok(())
}
