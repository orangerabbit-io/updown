//! Subcommands for the `nodes` resource (`GET /api/nodes`).

use anyhow::Result;
use clap::Subcommand;
use std::collections::HashMap;

use crate::output::{self, OutputMode};
use updown_lib::client::Client;
use updown_lib::models::node::{Node, NodeRow};

/// Actions available under `updown nodes`.
#[derive(Subcommand)]
pub enum NodesAction {
    /// List monitoring node locations
    List,
    /// List monitoring node IP addresses
    Ips {
        #[arg(long, conflicts_with = "ipv6")]
        ipv4: bool,
        #[arg(long, conflicts_with = "ipv4")]
        ipv6: bool,
        #[arg(long, value_parser = ["json", "txt"])]
        format: Option<String>,
    },
}

/// Dispatches the requested nodes action to the appropriate API call.
pub fn run(action: NodesAction, client: &Client, mode: OutputMode) -> Result<()> {
    match action {
        NodesAction::List => list(client, mode),
        NodesAction::Ips { ipv4, ipv6, format } => ips(client, mode, ipv4, ipv6, format),
    }
}

fn list(client: &Client, mode: OutputMode) -> Result<()> {
    let resp = client.get("/api/nodes")?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let nodes: HashMap<String, Node> = resp.json()?;
            let mut rows: Vec<NodeRow> = nodes
                .iter()
                .map(|(code, node)| NodeRow {
                    code: code.clone(),
                    city: node.city.clone().unwrap_or("-".to_string()),
                    country: node.country.clone().unwrap_or("-".to_string()),
                    ip: node.ip.clone().unwrap_or("-".to_string()),
                    ip6: node.ip6.clone().unwrap_or("-".to_string()),
                })
                .collect();
            rows.sort_by(|a, b| a.code.cmp(&b.code));
            output::print_table(&rows);
        }
        OutputMode::Axi => {
            let nodes: HashMap<String, Node> = resp.json()?;
            let country_count = nodes
                .values()
                .filter_map(|n| n.country_code.as_ref())
                .collect::<std::collections::HashSet<_>>()
                .len();
            let aggregate = format!("{} nodes across {} countries", nodes.len(), country_count);
            let mut rows: Vec<NodeRow> = nodes
                .iter()
                .map(|(code, node)| NodeRow {
                    code: code.clone(),
                    city: node.city.clone().unwrap_or_else(|| "-".to_string()),
                    country: node.country.clone().unwrap_or_else(|| "-".to_string()),
                    ip: node.ip.clone().unwrap_or_else(|| "-".to_string()),
                    ip6: node.ip6.clone().unwrap_or_else(|| "-".to_string()),
                })
                .collect();
            rows.sort_by(|a, b| a.code.cmp(&b.code));
            output::print_axi_list(
                &rows,
                "nodes",
                &aggregate,
                &["nodes ips --ipv4", "nodes ips --ipv6"],
            );
        }
    }

    Ok(())
}

fn ips(
    client: &Client,
    mode: OutputMode,
    ipv4: bool,
    ipv6: bool,
    format: Option<String>,
) -> Result<()> {
    let path = if ipv4 {
        "/api/nodes/ipv4"
    } else if ipv6 {
        "/api/nodes/ipv6"
    } else {
        "/api/nodes/ips"
    };

    // If --format is explicitly set, dispatch based on format value
    if let Some(fmt) = &format {
        if fmt == "txt" {
            let resp = client.get_text_with_params(path, &[("format", "txt")])?;
            output::print_raw(&resp);
            return Ok(());
        }
        // --format json: use the API's json format param but go through normal output
        let resp = client.get_with_params(path, &[("format", "json")])?;
        let json: serde_json::Value = resp.json()?;
        output::print_json(&json);
        return Ok(());
    }

    let resp = client.get(path)?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let ips: Vec<String> = resp.json()?;
            for ip in &ips {
                println!("{}", ip);
            }
        }
        OutputMode::Axi => {
            let ips: Vec<String> = resp.json()?;
            for ip in &ips {
                println!("{}", ip);
            }
            println!("help[nodes list]");
        }
    }

    Ok(())
}
