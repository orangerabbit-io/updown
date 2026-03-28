//! Subcommands for the `checks` resource (`GET/POST/PUT/DELETE /api/checks`).

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::output::{self, OutputMode};
use updown_lib::client::Client;
use updown_lib::models::check::{Check, CheckRow, Downtime, DowntimeRow};

/// Actions available under `updown checks`.
#[derive(Subcommand)]
pub enum ChecksAction {
    /// List all checks
    List,
    /// Get a single check
    Get {
        /// Check token
        token: String,
        /// Include metrics
        #[arg(long)]
        metrics: bool,
    },
    /// Create a new check
    Create {
        /// URL to monitor (required for all types except pulse)
        url: Option<String>,
        #[arg(long, value_parser = ["https", "http", "icmp", "pulse", "tcp", "tcps"])]
        r#type: Option<String>,
        #[arg(long, value_parser = ["15", "30", "60", "120", "300", "600", "1800", "3600"])]
        period: Option<String>,
        #[arg(long, value_parser = ["0.125", "0.25", "0.5", "1.0", "2.0", "4.0", "8.0"])]
        apdex_t: Option<String>,
        #[arg(long)]
        enabled: bool,
        #[arg(long)]
        published: bool,
        #[arg(long)]
        alias: Option<String>,
        #[arg(long)]
        string_match: Option<String>,
        #[arg(long)]
        mute_until: Option<String>,
        #[arg(long, value_parser = ["GET/HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])]
        http_verb: Option<String>,
        #[arg(long)]
        http_body: Option<String>,
        #[arg(long, value_delimiter = ',')]
        disabled_locations: Option<Vec<String>>,
        #[arg(long, value_delimiter = ',')]
        recipients: Option<Vec<String>>,
        #[arg(long)]
        custom_headers: Option<String>,
    },
    /// Update an existing check
    Update {
        /// Check token
        token: String,
        #[arg(long)]
        url: Option<String>,
        #[arg(long, value_parser = ["https", "http", "icmp", "pulse", "tcp", "tcps"])]
        r#type: Option<String>,
        #[arg(long, value_parser = ["15", "30", "60", "120", "300", "600", "1800", "3600"])]
        period: Option<String>,
        #[arg(long, value_parser = ["0.125", "0.25", "0.5", "1.0", "2.0", "4.0", "8.0"])]
        apdex_t: Option<String>,
        #[arg(long)]
        enabled: Option<bool>,
        #[arg(long)]
        published: Option<bool>,
        #[arg(long)]
        alias: Option<String>,
        #[arg(long)]
        string_match: Option<String>,
        #[arg(long)]
        mute_until: Option<String>,
        #[arg(long, value_parser = ["GET/HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])]
        http_verb: Option<String>,
        #[arg(long)]
        http_body: Option<String>,
        #[arg(long, value_delimiter = ',')]
        disabled_locations: Option<Vec<String>>,
        #[arg(long, value_delimiter = ',')]
        recipients: Option<Vec<String>>,
        #[arg(long)]
        custom_headers: Option<String>,
    },
    /// Delete a check
    Delete {
        /// Check token
        token: String,
    },
    /// View downtime history
    Downtimes {
        /// Check token
        token: String,
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        results: bool,
    },
    /// View performance metrics
    Metrics {
        /// Check token
        token: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, value_parser = ["time", "host"])]
        group: Option<String>,
    },
}

/// Fields shared by the create and update commands.
struct CheckBodyParams<'a> {
    url: &'a Option<String>,
    r#type: &'a Option<String>,
    period: &'a Option<String>,
    apdex_t: &'a Option<String>,
    alias: &'a Option<String>,
    string_match: &'a Option<String>,
    mute_until: &'a Option<String>,
    http_verb: &'a Option<String>,
    http_body: &'a Option<String>,
    disabled_locations: &'a Option<Vec<String>>,
    recipients: &'a Option<Vec<String>>,
    custom_headers: &'a Option<String>,
}

/// Builds the request body shared by the create and update arms.
fn build_check_body(
    p: &CheckBodyParams<'_>,
) -> Result<std::collections::HashMap<String, serde_json::Value>> {
    let mut body = std::collections::HashMap::new();
    if let Some(u) = p.url {
        body.insert("url".to_string(), serde_json::Value::String(u.clone()));
    }
    if let Some(t) = p.r#type {
        body.insert("type".to_string(), serde_json::Value::String(t.clone()));
    }
    if let Some(period) = p.period {
        let p_num: u32 = period.parse().expect("validated by clap value_parser");
        body.insert("period".to_string(), serde_json::json!(p_num));
    }
    if let Some(a) = p.apdex_t {
        let a_num: f64 = a.parse().expect("validated by clap value_parser");
        body.insert("apdex_t".to_string(), serde_json::json!(a_num));
    }
    if let Some(a) = p.alias {
        body.insert("alias".to_string(), serde_json::Value::String(a.clone()));
    }
    if let Some(s) = p.string_match {
        body.insert(
            "string_match".to_string(),
            serde_json::Value::String(s.clone()),
        );
    }
    if let Some(m) = p.mute_until {
        body.insert(
            "mute_until".to_string(),
            serde_json::Value::String(m.clone()),
        );
    }
    if let Some(v) = p.http_verb {
        body.insert(
            "http_verb".to_string(),
            serde_json::Value::String(v.clone()),
        );
    }
    if let Some(b) = p.http_body {
        body.insert(
            "http_body".to_string(),
            serde_json::Value::String(b.clone()),
        );
    }
    if let Some(locs) = p.disabled_locations {
        body.insert("disabled_locations".to_string(), serde_json::json!(locs));
    }
    if let Some(recs) = p.recipients {
        body.insert("recipients".to_string(), serde_json::json!(recs));
    }
    if let Some(hdrs) = p.custom_headers {
        let parsed: serde_json::Value =
            serde_json::from_str(hdrs).context("--custom-headers must be valid JSON")?;
        body.insert("custom_headers".to_string(), parsed);
    }
    Ok(body)
}

/// Dispatches the requested checks action to the appropriate API call.
pub fn run(action: ChecksAction, client: &Client, mode: OutputMode) -> Result<()> {
    match action {
        ChecksAction::List => list(client, mode),
        ChecksAction::Get { token, metrics } => get(client, mode, &token, metrics),
        ChecksAction::Create {
            url,
            r#type,
            period,
            apdex_t,
            enabled,
            published,
            alias,
            string_match,
            mute_until,
            http_verb,
            http_body,
            disabled_locations,
            recipients,
            custom_headers,
        } => {
            // Validate url is provided unless type is pulse
            let is_pulse = r#type.as_deref() == Some("pulse");
            if !is_pulse && url.is_none() {
                anyhow::bail!("URL is required for all check types except pulse");
            }

            let mut body = build_check_body(&CheckBodyParams {
                url: &url,
                r#type: &r#type,
                period: &period,
                apdex_t: &apdex_t,
                alias: &alias,
                string_match: &string_match,
                mute_until: &mute_until,
                http_verb: &http_verb,
                http_body: &http_body,
                disabled_locations: &disabled_locations,
                recipients: &recipients,
                custom_headers: &custom_headers,
            })?;

            if enabled {
                body.insert("enabled".to_string(), serde_json::json!(true));
            }
            if published {
                body.insert("published".to_string(), serde_json::json!(true));
            }

            let resp = client.post("/api/checks", &body)?;

            match mode {
                OutputMode::Json => {
                    let json: serde_json::Value = resp.json()?;
                    output::print_json(&json);
                }
                OutputMode::Table => {
                    let check: Check = resp.json()?;
                    output::print_confirm(&format!(
                        "Check created: {} ({})",
                        check.token, check.url
                    ));
                }
                OutputMode::Axi => {
                    let check: Check = resp.json()?;
                    let h1 = format!("checks get {}", check.token);
                    output::print_axi_confirm(
                        &format!("Check created: {} ({})", check.token, check.url),
                        &[&h1, "checks list"],
                    );
                }
            }

            Ok(())
        }
        ChecksAction::Update {
            token,
            url,
            r#type,
            period,
            apdex_t,
            enabled,
            published,
            alias,
            string_match,
            mute_until,
            http_verb,
            http_body,
            disabled_locations,
            recipients,
            custom_headers,
        } => {
            let mut body = build_check_body(&CheckBodyParams {
                url: &url,
                r#type: &r#type,
                period: &period,
                apdex_t: &apdex_t,
                alias: &alias,
                string_match: &string_match,
                mute_until: &mute_until,
                http_verb: &http_verb,
                http_body: &http_body,
                disabled_locations: &disabled_locations,
                recipients: &recipients,
                custom_headers: &custom_headers,
            })?;

            if let Some(e) = enabled {
                body.insert("enabled".to_string(), serde_json::json!(e));
            }
            if let Some(p) = published {
                body.insert("published".to_string(), serde_json::json!(p));
            }

            let resp = client.put(&format!("/api/checks/{}", token), &body)?;

            match mode {
                OutputMode::Json => {
                    let json: serde_json::Value = resp.json()?;
                    output::print_json(&json);
                }
                OutputMode::Table => {
                    let check: Check = resp.json()?;
                    output::print_confirm(&format!(
                        "Check updated: {} ({})",
                        check.token, check.url
                    ));
                }
                OutputMode::Axi => {
                    let check: Check = resp.json()?;
                    let h1 = format!("checks get {}", check.token);
                    output::print_axi_confirm(
                        &format!("Check updated: {} ({})", check.token, check.url),
                        &[&h1, "checks list"],
                    );
                }
            }

            Ok(())
        }
        ChecksAction::Delete { token } => {
            let resp = client.delete(&format!("/api/checks/{}", token))?;

            match mode {
                OutputMode::Json => {
                    let json: serde_json::Value = resp.json()?;
                    output::print_json(&json);
                }
                OutputMode::Table => {
                    output::print_confirm(&format!("Deleted check {}", token));
                }
                OutputMode::Axi => {
                    output::print_axi_confirm(
                        &format!("Deleted check {}", token),
                        &["checks list"],
                    );
                }
            }

            Ok(())
        }
        ChecksAction::Downtimes {
            token,
            page,
            results,
        } => {
            let mut params = Vec::new();
            let page_str;
            if let Some(p) = page {
                page_str = p.to_string();
                params.push(("page", page_str.as_str()));
            }
            if results {
                params.push(("results", "true"));
            }

            let path = format!("/api/checks/{}/downtimes", token);
            let resp = client.get_with_params(&path, &params)?;

            match mode {
                OutputMode::Json => {
                    let json: serde_json::Value = resp.json()?;
                    output::print_json(&json);
                }
                OutputMode::Table => {
                    let downtimes: Vec<Downtime> = resp.json()?;
                    let rows: Vec<DowntimeRow> = downtimes.iter().map(DowntimeRow::from).collect();
                    output::print_table(&rows);
                }
                OutputMode::Axi => {
                    let downtimes: Vec<Downtime> = resp.json()?;
                    let total_duration: u64 = downtimes.iter().filter_map(|d| d.duration).sum();
                    let aggregate = format!(
                        "{} downtimes, total_duration: {}s",
                        downtimes.len(),
                        total_duration
                    );
                    let rows: Vec<DowntimeRow> =
                        downtimes.iter().map(DowntimeRow::from).collect();
                    let h1 = format!("checks get {}", token);
                    let h2 = format!("checks metrics {}", token);
                    output::print_axi_list(&rows, "downtimes", &aggregate, &[&h1, &h2]);
                }
            }

            Ok(())
        }
        ChecksAction::Metrics {
            token,
            from,
            to,
            group,
        } => {
            let mut params = Vec::new();
            if let Some(f) = &from {
                params.push(("from", f.as_str()));
            }
            if let Some(t) = &to {
                params.push(("to", t.as_str()));
            }
            if let Some(g) = &group {
                params.push(("group", g.as_str()));
            }

            let path = format!("/api/checks/{}/metrics", token);
            let resp = client.get_with_params(&path, &params)?;

            match mode {
                OutputMode::Json => {
                    let json: serde_json::Value = resp.json()?;
                    output::print_json(&json);
                }
                OutputMode::Table => {
                    // Metrics are a complex nested structure — display as formatted JSON
                    // even in table mode, since they don't map well to a flat table
                    let json: serde_json::Value = resp.json()?;
                    output::print_json(&json);
                }
                OutputMode::Axi => {
                    let json: serde_json::Value = resp.json()?;
                    let apdex = json
                        .get("apdex")
                        .and_then(|v| v.as_f64())
                        .map(|a| format!("{:.4}", a))
                        .unwrap_or_else(|| "-".to_string());
                    let h1 = format!("checks get {}", token);
                    let h2 = format!("checks downtimes {}", token);
                    let mut out = format!("summary: apdex: {}\n", apdex);
                    out.push_str(&output::format_axi_detail(&json, &[&h1, &h2]));
                    print!("{}", out);
                }
            }

            Ok(())
        }
    }
}

fn list(client: &Client, mode: OutputMode) -> Result<()> {
    let resp = client.get("/api/checks")?;

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let checks: Vec<Check> = resp.json()?;
            let rows: Vec<CheckRow> = checks.iter().map(CheckRow::from).collect();
            output::print_table(&rows);
        }
        OutputMode::Axi => {
            let checks: Vec<Check> = resp.json()?;
            let down_count = checks.iter().filter(|c| c.down == Some(true)).count();
            let worst_uptime = checks
                .iter()
                .filter_map(|c| c.uptime)
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|u| format!("{:.2}%", u))
                .unwrap_or_else(|| "-".to_string());
            let aggregate = format!(
                "{} checks, {} down, worst_uptime: {}",
                checks.len(),
                down_count,
                worst_uptime
            );
            let rows: Vec<CheckRow> = checks.iter().map(CheckRow::from).collect();
            output::print_axi_list(
                &rows,
                "checks",
                &aggregate,
                &["checks get <token> --metrics", "checks downtimes <token>"],
            );
        }
    }

    Ok(())
}

fn get(client: &Client, mode: OutputMode, token: &str, metrics: bool) -> Result<()> {
    let path = format!("/api/checks/{}", token);
    let resp = if metrics {
        client.get_with_params(&path, &[("metrics", "true")])?
    } else {
        client.get(&path)?
    };

    match mode {
        OutputMode::Json => {
            let json: serde_json::Value = resp.json()?;
            output::print_json(&json);
        }
        OutputMode::Table => {
            let check: Check = resp.json()?;
            let mut pairs = vec![
                ("Token", check.token.clone()),
                ("URL", check.url.clone()),
                (
                    "Status",
                    if check.down == Some(true) {
                        "down".to_string()
                    } else {
                        "up".to_string()
                    },
                ),
                (
                    "Uptime",
                    check
                        .uptime
                        .map(|u| format!("{:.2}%", u))
                        .unwrap_or("-".to_string()),
                ),
                (
                    "Period",
                    check
                        .period
                        .map(|p| format!("{}s", p))
                        .unwrap_or("-".to_string()),
                ),
                ("Type", check.check_type.clone().unwrap_or("-".to_string())),
                (
                    "Enabled",
                    check
                        .enabled
                        .map(|e| e.to_string())
                        .unwrap_or("-".to_string()),
                ),
            ];

            if let Some(alias) = &check.alias {
                pairs.insert(2, ("Alias", alias.clone()));
            }

            if metrics {
                if let Some(m) = &check.metrics {
                    pairs.push((
                        "Apdex",
                        m.apdex
                            .map(|a| format!("{:.4}", a))
                            .unwrap_or("-".to_string()),
                    ));
                }
            }

            let pair_refs: Vec<(&str, String)> = pairs.into_iter().collect();
            output::print_kv(&pair_refs);
        }
        OutputMode::Axi => {
            let check: Check = resp.json()?;
            let h1 = format!("checks update {}", token);
            let h2 = format!("checks downtimes {}", token);
            let h3 = format!("checks metrics {}", token);
            output::print_axi_detail(&check, &[&h1, &h2, &h3]);
        }
    }

    Ok(())
}
