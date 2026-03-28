# AXI Output Mode Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add AXI-compliant TOON output as the default CLI mode, optimizing for AI agent consumption with aggregates, help hints, structured errors, and content truncation.

**Architecture:** Third `OutputMode::Axi` variant added to existing output layer. `toon-format` crate handles TOON serialization. Row types gain `Serialize` derive. Client gains typed `ApiError` for structured error handling. Command handlers compute aggregates and pass hints to new `print_axi_*` functions.

**Tech Stack:** Rust, toon-format crate, serde, clap (mutually exclusive flag groups)

**Spec:** `docs/superpowers/specs/2026-03-28-axi-output-mode-design.md`

---

## Chunk 1: Foundation (OutputMode, TOON dependency, Row Serialize)

### Task 1: Add `toon-format` dependency and `Serialize` to Row types

**Files:**
- Modify: `updown/Cargo.toml:13-18`
- Modify: `updown-lib/src/models/check.rs:71,151`
- Modify: `updown-lib/src/models/node.rs:39`
- Modify: `updown-lib/src/models/recipient.rs:30`
- Modify: `updown-lib/src/models/status_page.rs:33`

- [ ] **Step 1: Write failing test — CheckRow serializes to JSON (proves Serialize works)**

In `updown-lib/src/models/check.rs`, add to the existing `tests` module:

```rust
#[test]
fn test_check_row_serialize() {
    let row = CheckRow {
        token: "abc".to_string(),
        status: "up".to_string(),
        url: "https://example.com".to_string(),
        uptime: "99.97%".to_string(),
        apdex: "0.50".to_string(),
        period: "60s".to_string(),
    };
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("abc"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p updown-lib test_check_row_serialize`
Expected: FAIL — `Serialize` is not implemented for `CheckRow`

- [ ] **Step 3: Add `Serialize` derive to all Row types**

In `updown-lib/src/models/check.rs` line 71, change:
```rust
#[derive(Debug, Tabled)]
```
to:
```rust
#[derive(Debug, Serialize, Tabled)]
```

Do the same for:
- `DowntimeRow` at line 151
- `NodeRow` in `node.rs` line 39
- `RecipientRow` in `recipient.rs` line 30
- `StatusPageRow` in `status_page.rs` line 33

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p updown-lib test_check_row_serialize`
Expected: PASS

- [ ] **Step 5: Add `toon-format` dependency to CLI crate**

In `updown/Cargo.toml`, add to `[dependencies]`:
```toml
toon-format = "0.4"
```

- [ ] **Step 6: Verify workspace builds**

Run: `cargo build --workspace`
Expected: Compiles without errors

- [ ] **Step 7: Commit**

```bash
git add updown/Cargo.toml updown-lib/src/models/check.rs updown-lib/src/models/node.rs updown-lib/src/models/recipient.rs updown-lib/src/models/status_page.rs
git commit -m "feat: add Serialize to Row types, add toon-format dependency"
```

---

### Task 2: Refactor OutputMode to three variants with CLI flag changes

**Files:**
- Modify: `updown/src/output.rs:1-83`
- Modify: `updown/src/main.rs:26-39,92`
- Test: `updown/src/output.rs` (inline tests)

- [ ] **Step 1: Write failing test — OutputMode::Axi is the default**

In `updown/src/output.rs`, replace the existing test with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_mode_default_is_axi() {
        assert_eq!(OutputMode::from_flags(false, false), OutputMode::Axi);
    }

    #[test]
    fn test_output_mode_json_flag() {
        assert_eq!(OutputMode::from_flags(true, false), OutputMode::Json);
    }

    #[test]
    fn test_output_mode_table_flag() {
        assert_eq!(OutputMode::from_flags(false, true), OutputMode::Table);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p updown test_output_mode_default_is_axi`
Expected: FAIL — `from_flags` does not exist, `Axi` variant does not exist

- [ ] **Step 3: Update OutputMode enum and constructor**

In `updown/src/output.rs`, replace the `OutputMode` enum and impl block (lines 10-28):

```rust
/// Controls how output is rendered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    /// Token-efficient TOON format with aggregates and help hints (default).
    Axi,
    /// Columnar table output suitable for human reading.
    Table,
    /// Pretty-printed JSON suitable for piping or scripting.
    Json,
}

impl OutputMode {
    /// Determines output mode from CLI flags. No flags = Axi (default).
    pub fn from_flags(json: bool, table: bool) -> Self {
        if json {
            OutputMode::Json
        } else if table {
            OutputMode::Table
        } else {
            OutputMode::Axi
        }
    }
}
```

- [ ] **Step 4: Update CLI struct in main.rs**

In `updown/src/main.rs`, replace the `Cli` struct (lines 26-39):

```rust
/// Top-level CLI entry point parsed by clap.
#[derive(Parser)]
#[command(name = "updown", about = "CLI for the updown.io monitoring API")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output as JSON
    #[arg(long, global = true, conflicts_with = "table")]
    pub json: bool,

    /// Output as human-readable table
    #[arg(long, global = true, conflicts_with = "json")]
    pub table: bool,

    /// API key (overrides config file and env var)
    #[arg(long, global = true)]
    pub api_key: Option<String>,
}
```

Update line 92 in `run()`:
```rust
let mode = output::OutputMode::from_flags(cli.json, cli.table);
```

Update the module doc comment at line 7:
```rust
//! pass `--json` for machine-readable JSON output or `--table` for a human-readable table.
```

- [ ] **Step 5: Add `Axi` match arms to all command handlers (passthrough to Table for now)**

In each handler file, every `match mode` block needs an `Axi` arm. For now, make `Axi` behave identically to `Table` so the code compiles. We'll replace these with real AXI output in later tasks.

In `updown/src/cmd/checks.rs`, for every `match mode { ... }` block, add an `OutputMode::Axi` arm that duplicates the `OutputMode::Table` behavior. There are 8 match blocks in this file:
- `list()` (line 402)
- `get()` (line 425)
- `Create` arm (line 241)
- `Update` arm (line 298)
- `Delete` arm (line 317)
- `Downtimes` arm (line 347)
- `Metrics` arm (line 381)

For example, `list()` becomes:
```rust
match mode {
    OutputMode::Json => {
        let json: serde_json::Value = resp.json()?;
        output::print_json(&json);
    }
    OutputMode::Table | OutputMode::Axi => {
        let checks: Vec<Check> = resp.json()?;
        let rows: Vec<CheckRow> = checks.iter().map(CheckRow::from).collect();
        output::print_table(&rows);
    }
}
```

Apply the same `OutputMode::Table | OutputMode::Axi` pattern to all match blocks in:
- `updown/src/cmd/nodes.rs` (3 match blocks: `list`, `ips` with format, `ips` without format)
- `updown/src/cmd/recipients.rs` (3 match blocks: `list`, `create`, `delete`)
- `updown/src/cmd/status_pages.rs` (4 match blocks: `list`, `create`, `update`, `delete`)

- [ ] **Step 6: Update existing integration tests that relied on default=Table**

Tests that don't pass `--json` now get AXI mode. While AXI is currently a passthrough to Table, this will break once real AXI output is wired in later tasks. Proactively add `--table` to tests that validate table-specific output:

In `updown/tests/checks_test.rs`:
- `test_checks_list_table`: add `"--table"` to args (the test name already implies it tests table output)
- `test_checks_get`: add `"--table"` to args
- `test_checks_delete`: add `"--table"` to args

Apply the same pattern in `nodes_test.rs`, `recipients_test.rs`, `status_pages_test.rs` — any test not using `--json` should use `--table` to explicitly test table mode.

- [ ] **Step 7: Run all tests**

Run: `cargo test --workspace`
Expected: All tests pass.

- [ ] **Step 8: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Clean

- [ ] **Step 9: Commit**

```bash
git add updown/src/output.rs updown/src/main.rs updown/src/cmd/ updown/tests/
git commit -m "refactor: add OutputMode::Axi variant, make it default"
```

---

## Chunk 2: AXI Output Functions (TOON, truncation, help, errors)

### Task 3: Implement TOON output functions and truncation in output.rs

**Files:**
- Modify: `updown/src/output.rs`
- Test: `updown/src/output.rs` (inline tests)

- [ ] **Step 1: Write failing test — truncate_field**

Add to `updown/src/output.rs` tests module:

```rust
#[test]
fn test_truncate_field_short() {
    assert_eq!(truncate_field("hello", 200), "hello");
}

#[test]
fn test_truncate_field_exact() {
    let s = "a".repeat(200);
    assert_eq!(truncate_field(&s, 200), s);
}

#[test]
fn test_truncate_field_long() {
    let s = "a".repeat(300);
    let result = truncate_field(&s, 200);
    assert!(result.len() < 300);
    assert!(result.ends_with("[300]"));
    assert!(result.contains("..."));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p updown test_truncate_field`
Expected: FAIL — `truncate_field` not found

- [ ] **Step 3: Implement truncate_field**

Add to `updown/src/output.rs`:

```rust
/// Truncates a string field to `max` characters for AXI output.
/// Fields exceeding `max` are cut to `max - suffix_len` and appended with `...[{original_len}]`.
pub fn truncate_field(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    let total = value.len();
    let suffix = format!("...[{}]", total);
    let keep = max - suffix.len();
    // Find a safe UTF-8 boundary at or before `keep`
    let safe_keep = value
        .char_indices()
        .take_while(|(i, _)| *i <= keep)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}{}", &value[..safe_keep], suffix)
}
```

- [ ] **Step 4: Run truncation tests**

Run: `cargo test -p updown test_truncate_field`
Expected: PASS

- [ ] **Step 5: Write failing test — print_axi_list produces TOON with summary and help**

```rust
#[test]
fn test_print_axi_list_format() {
    // Capture stdout would be complex; test the format_axi_list helper instead
    let rows = vec![
        serde_json::json!({"token": "abc", "status": "up"}),
        serde_json::json!({"token": "def", "status": "down"}),
    ];
    let output = format_axi_list(&rows, "checks", "2 checks, 1 down", &["checks get <token>"]);
    assert!(output.contains("summary: 2 checks, 1 down"));
    assert!(output.contains("help[checks get <token>]"));
}

#[test]
fn test_print_axi_list_empty() {
    let rows: Vec<serde_json::Value> = vec![];
    let output = format_axi_list(&rows, "checks", "0 checks", &["checks create --url <url>"]);
    assert!(output.contains("summary: 0 checks"));
    assert!(output.contains("checks[0]"));
    assert!(output.contains("help[checks create --url <url>]"));
}
```

- [ ] **Step 6: Run tests to verify they fail**

Run: `cargo test -p updown test_print_axi_list`
Expected: FAIL — `format_axi_list` not found

- [ ] **Step 7: Implement AXI output functions**

Add to `updown/src/output.rs`:

```rust
use serde::Serialize;
use toon_format::encode_default;

/// Formats a list of items as AXI output: summary + TOON + help hints.
pub fn format_axi_list<T: Serialize>(
    items: &[T],
    resource_name: &str,
    aggregate: &str,
    hints: &[&str],
) -> String {
    let mut out = format!("summary: {}\n", aggregate);

    if items.is_empty() {
        out.push_str(&format!("{}[0]:\n", resource_name));
    } else {
        match encode_default(&items) {
            Ok(toon) => out.push_str(&toon),
            Err(_) => out.push_str(&format!("{}[{}]: <encoding error>\n", resource_name, items.len())),
        }
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }

    if !hints.is_empty() {
        out.push_str(&format!("help[{}]", hints.join(", ")));
    }
    out
}

/// Formats a single item detail as AXI output: TOON + help hints.
pub fn format_axi_detail<T: Serialize>(item: &T, hints: &[&str]) -> String {
    let mut out = match encode_default(&item) {
        Ok(toon) => toon,
        Err(_) => "<encoding error>".to_string(),
    };
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !hints.is_empty() {
        out.push_str(&format!("help[{}]", hints.join(", ")));
    }
    out
}

/// Formats a confirmation message as AXI output with help hints.
pub fn format_axi_confirm(message: &str, hints: &[&str]) -> String {
    let mut out = format!("{}\n", message);
    if !hints.is_empty() {
        out.push_str(&format!("help[{}]", hints.join(", ")));
    }
    out
}

/// Formats a structured error as AXI output (printed to stdout with exit 0).
pub fn format_axi_error(code: u16, message: &str) -> String {
    format!("error{{code,message}}:\n  {},{}", code, message)
}

/// Prints AXI list output to stdout.
pub fn print_axi_list<T: Serialize>(
    items: &[T],
    resource_name: &str,
    aggregate: &str,
    hints: &[&str],
) {
    print!("{}", format_axi_list(items, resource_name, aggregate, hints));
}

/// Prints AXI detail output to stdout.
pub fn print_axi_detail<T: Serialize>(item: &T, hints: &[&str]) {
    print!("{}", format_axi_detail(item, hints));
}

/// Prints AXI confirmation to stdout.
pub fn print_axi_confirm(message: &str, hints: &[&str]) {
    print!("{}", format_axi_confirm(message, hints));
}

/// Prints AXI error to stdout.
pub fn print_axi_error(code: u16, message: &str) {
    print!("{}", format_axi_error(code, message));
}
```

- [ ] **Step 8: Write test for format_axi_error**

```rust
#[test]
fn test_format_axi_error() {
    let output = format_axi_error(401, "Authentication failed.");
    assert!(output.contains("error{code,message}:"));
    assert!(output.contains("401,Authentication failed."));
}

#[test]
fn test_format_axi_confirm() {
    let output = format_axi_confirm("Check created: abc1", &["checks get abc1", "checks list"]);
    assert!(output.contains("Check created: abc1"));
    assert!(output.contains("help[checks get abc1, checks list]"));
}

#[test]
fn test_format_axi_detail() {
    let item = serde_json::json!({"token": "abc", "url": "https://example.com"});
    let output = format_axi_detail(&item, &["checks update abc"]);
    assert!(output.contains("help[checks update abc]"));
}
```

- [ ] **Step 9: Run all output tests**

Run: `cargo test -p updown -- output::tests`
Expected: All PASS

- [ ] **Step 10: Commit**

```bash
git add updown/src/output.rs
git commit -m "feat: add AXI output functions (TOON, truncation, help, errors)"
```

---

## Chunk 3: Typed API Errors in Client

### Task 4: Add ApiError enum to client for structured error handling

**Files:**
- Modify: `updown-lib/src/client.rs`
- Test: `updown-lib/src/client.rs` (inline tests or new test)

- [ ] **Step 1: Write failing test — ApiError can be extracted from anyhow error**

Add to `updown-lib/src/client.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_downcast() {
        let err = ApiError {
            status_code: 401,
            message: "Authentication failed".to_string(),
        };
        let anyhow_err: anyhow::Error = err.into();
        let downcast = anyhow_err.downcast_ref::<ApiError>().unwrap();
        assert_eq!(downcast.status_code, 401);
        assert_eq!(downcast.message, "Authentication failed");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p updown-lib test_api_error_downcast`
Expected: FAIL — `ApiError` not found

- [ ] **Step 3: Implement ApiError**

Add above the `Client` struct in `updown-lib/src/client.rs`:

```rust
use std::fmt;

/// A known API error that can be displayed as structured output in AXI mode.
#[derive(Debug)]
pub struct ApiError {
    /// HTTP status code (401, 403, 404, 422, 429).
    pub status_code: u16,
    /// Human-readable error message.
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (HTTP {})", self.message, self.status_code)
    }
}

impl std::error::Error for ApiError {}
```

- [ ] **Step 4: Refactor check_status to use ApiError**

Replace `check_status` (lines 140-154):

```rust
fn check_status(resp: Response) -> Result<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let url = resp.url().to_string();
    let body = resp.text().unwrap_or_default();
    let code = status.as_u16();
    let message = match code {
        401 | 403 => format!("Authentication failed (HTTP {}): {}", status, body),
        404 => format!("Not found (HTTP {}): {}", status, body),
        422 => format!("Validation error (HTTP {}): {}", status, body),
        429 => format!("Rate limited (HTTP {}): {}", status, body),
        _ => format!("API error (HTTP {}) for {}: {}", status, url, body),
    };
    Err(ApiError { status_code: code, message }.into())
}
```

- [ ] **Step 5: Run all tests**

Run: `cargo test --workspace`
Expected: All pass — behavior is identical, just using typed error now

- [ ] **Step 6: Commit**

```bash
git add updown-lib/src/client.rs
git commit -m "refactor: add typed ApiError for structured error handling"
```

---

## Chunk 4: Wire Up AXI in Command Handlers

### Task 5: Implement AXI output for checks commands

**Files:**
- Modify: `updown/src/cmd/checks.rs`
- Create: `updown/tests/fixtures/checks_list_multi.json`
- Test: `updown/tests/checks_test.rs`

- [ ] **Step 1: Write integration test — checks list default (AXI) output**

Create fixture `updown/tests/fixtures/checks_list_multi.json`:
```json
[
  {
    "token": "abc123",
    "url": "https://api.example.com",
    "uptime": 99.97,
    "down": false,
    "period": 60,
    "apdex_t": 0.5,
    "type": "https"
  },
  {
    "token": "def456",
    "url": "https://web.example.com",
    "uptime": 98.50,
    "down": true,
    "period": 30,
    "apdex_t": 1.0,
    "type": "https"
  }
]
```

Add to `updown/tests/checks_test.rs`:
```rust
#[test]
fn test_checks_list_axi_default() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("checks_list_multi.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "checks", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("summary:"))
        .stdout(predicate::str::contains("2 checks"))
        .stdout(predicate::str::contains("1 down"))
        .stdout(predicate::str::contains("help["));

    mock.assert();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p updown test_checks_list_axi_default`
Expected: FAIL — default output is still Table-like, no "summary:" line

- [ ] **Step 3: Implement AXI output for checks list**

In `updown/src/cmd/checks.rs`, replace `list()`:

```rust
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
                .unwrap_or("-".to_string());
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
```

- [ ] **Step 4: Run test**

Run: `cargo test -p updown test_checks_list_axi_default`
Expected: PASS

- [ ] **Step 5: Implement AXI for remaining checks subcommands**

Replace the `OutputMode::Table | OutputMode::Axi` arms in each match block with separate `Axi` arms:

**checks get:**
```rust
OutputMode::Axi => {
    let check: Check = resp.json()?;
    let hints = &[
        &format!("checks update {}", token) as &str,
        &format!("checks downtimes {}", token),
        &format!("checks metrics {}", token),
    ];
    // For detail view, serialize the full Check model
    output::print_axi_detail(&check, hints);
}
```

Note: the `hints` variable needs owned strings. Use a `let` binding pattern:
```rust
OutputMode::Axi => {
    let check: Check = resp.json()?;
    let h1 = format!("checks update {}", token);
    let h2 = format!("checks downtimes {}", token);
    let h3 = format!("checks metrics {}", token);
    output::print_axi_detail(&check, &[&h1, &h2, &h3]);
}
```

**checks create:**
```rust
OutputMode::Axi => {
    let check: Check = resp.json()?;
    let h1 = format!("checks get {}", check.token);
    output::print_axi_confirm(
        &format!("Check created: {} ({})", check.token, check.url),
        &[&h1, "checks list"],
    );
}
```

**checks update:**
```rust
OutputMode::Axi => {
    let check: Check = resp.json()?;
    let h1 = format!("checks get {}", check.token);
    output::print_axi_confirm(
        &format!("Check updated: {} ({})", check.token, check.url),
        &[&h1, "checks list"],
    );
}
```

**checks delete:**
```rust
OutputMode::Axi => {
    let _json: serde_json::Value = resp.json()?;
    output::print_axi_confirm(
        &format!("Deleted check {}", token),
        &["checks list"],
    );
}
```

**checks downtimes:**
```rust
OutputMode::Axi => {
    let downtimes: Vec<Downtime> = resp.json()?;
    let total_duration: u64 = downtimes.iter().filter_map(|d| d.duration).sum();
    let aggregate = format!(
        "{} downtimes, total_duration: {}s",
        downtimes.len(),
        total_duration
    );
    let rows: Vec<DowntimeRow> = downtimes.iter().map(DowntimeRow::from).collect();
    let h1 = format!("checks get {}", token);
    let h2 = format!("checks metrics {}", token);
    output::print_axi_list(&rows, "downtimes", &aggregate, &[&h1, &h2]);
}
```

**checks metrics:**

Metrics are opaque nested JSON. Use `format_axi_detail` but prepend a summary line manually:
```rust
OutputMode::Axi => {
    let json: serde_json::Value = resp.json()?;
    let apdex = json.get("apdex")
        .and_then(|v| v.as_f64())
        .map(|a| format!("{:.4}", a))
        .unwrap_or("-".to_string());
    let h1 = format!("checks get {}", token);
    let h2 = format!("checks downtimes {}", token);
    let mut out = format!("summary: apdex: {}\n", apdex);
    out.push_str(&output::format_axi_detail(&json, &[&h1, &h2]));
    print!("{}", out);
}
```

- [ ] **Step 6: Run all tests**

Run: `cargo test --workspace`
Expected: All pass

- [ ] **Step 7: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: Clean

- [ ] **Step 8: Commit**

```bash
git add updown/src/cmd/checks.rs updown/tests/checks_test.rs updown/tests/fixtures/checks_list_multi.json
git commit -m "feat: implement AXI output for checks commands"
```

---

### Task 6: Implement AXI output for nodes commands

**Files:**
- Modify: `updown/src/cmd/nodes.rs`
- Test: `updown/tests/nodes_test.rs`

- [ ] **Step 1: Write integration test — nodes list AXI default**

Add to `updown/tests/nodes_test.rs`:
```rust
#[test]
fn test_nodes_list_axi_default() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/nodes")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("nodes_list.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "nodes", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("summary:"))
        .stdout(predicate::str::contains("help["));

    mock.assert();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p updown test_nodes_list_axi_default`
Expected: FAIL

- [ ] **Step 3: Implement AXI for nodes list**

In `updown/src/cmd/nodes.rs`, replace `list()`:

```rust
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
            let aggregate = format!(
                "{} nodes across {} countries",
                nodes.len(),
                country_count
            );
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
```

- [ ] **Step 4: Implement AXI for nodes ips**

In the `ips()` function, there are two code paths:

**Early-return paths (lines 79-90):** When `--format` is specified, the function returns early before the `match mode` block. For AXI mode, `--format` still takes precedence (the user explicitly asked for a format), so no change needed here — the `--format` flag overrides output mode.

**Match block at line 94:** Add `Axi` arm:

```rust
OutputMode::Axi => {
    let ips: Vec<String> = resp.json()?;
    for ip in &ips {
        println!("{}", ip);
    }
    println!("help[nodes list]");
}
```

Raw IP list is already minimal text — pass through with help hint appended.

- [ ] **Step 5: Run test**

Run: `cargo test -p updown test_nodes_list_axi_default`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add updown/src/cmd/nodes.rs updown/tests/nodes_test.rs
git commit -m "feat: implement AXI output for nodes commands"
```

---

### Task 7: Implement AXI output for recipients commands

**Files:**
- Modify: `updown/src/cmd/recipients.rs`

- [ ] **Step 1: Implement AXI for recipients list**

```rust
OutputMode::Axi => {
    let recipients: Vec<Recipient> = resp.json()?;
    let mut type_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
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
```

- [ ] **Step 2: Implement AXI for recipients create**

```rust
OutputMode::Axi => {
    let r: Recipient = resp.json()?;
    output::print_axi_confirm(
        &format!("Recipient created: {} ({} {})", r.id, r.recipient_type, r.value.unwrap_or_default()),
        &["recipients list"],
    );
}
```

- [ ] **Step 3: Implement AXI for recipients delete**

```rust
OutputMode::Axi => {
    let _json: serde_json::Value = resp.json()?;
    output::print_axi_confirm(
        &format!("Deleted recipient {}", id),
        &["recipients list"],
    );
}
```

- [ ] **Step 4: Run all tests**

Run: `cargo test --workspace`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add updown/src/cmd/recipients.rs
git commit -m "feat: implement AXI output for recipients commands"
```

---

### Task 8: Implement AXI output for status-pages commands

**Files:**
- Modify: `updown/src/cmd/status_pages.rs`

- [ ] **Step 1: Implement AXI for status-pages list**

```rust
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
```

- [ ] **Step 2: Implement AXI for status-pages create**

```rust
OutputMode::Axi => {
    let sp: StatusPage = resp.json()?;
    output::print_axi_confirm(
        &format!("Status page created: {} ({})", sp.token, sp.url.unwrap_or_default()),
        &["status-pages list"],
    );
}
```

- [ ] **Step 3: Implement AXI for status-pages update**

```rust
OutputMode::Axi => {
    let sp: StatusPage = resp.json()?;
    output::print_axi_confirm(
        &format!("Status page updated: {}", sp.token),
        &["status-pages list"],
    );
}
```

- [ ] **Step 4: Implement AXI for status-pages delete**

```rust
OutputMode::Axi => {
    let _json: serde_json::Value = resp.json()?;
    output::print_axi_confirm(
        &format!("Deleted status page {}", token),
        &["status-pages list"],
    );
}
```

- [ ] **Step 5: Run all tests and clippy**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: All pass, clean clippy

- [ ] **Step 6: Commit**

```bash
git add updown/src/cmd/status_pages.rs
git commit -m "feat: implement AXI output for status-pages commands"
```

---

## Chunk 5: Structured Error Handling in Handlers + Content Truncation

### Task 9: Wire AXI error handling into command dispatch

**Files:**
- Modify: `updown/src/main.rs`

- [ ] **Step 1: Write integration test — 401 in AXI mode produces structured error on stdout**

Add to `updown/tests/checks_test.rs`:

```rust
#[test]
fn test_checks_list_axi_error_401() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks")
        .match_header("X-API-KEY", "bad-key")
        .with_status(401)
        .with_body("Unauthorized")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "bad-key", "checks", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()  // exit 0 in AXI mode
        .stdout(predicate::str::contains("error{code,message}:"))
        .stdout(predicate::str::contains("401"));

    mock.assert();
}

#[test]
fn test_checks_list_json_error_401() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks")
        .match_header("X-API-KEY", "bad-key")
        .with_status(401)
        .with_body("Unauthorized")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "bad-key", "--json", "checks", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .failure()  // exit 1 in JSON mode (unchanged behavior)
        .stderr(predicate::str::contains("Authentication failed"));

    mock.assert();
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p updown test_checks_list_axi_error_401`
Expected: FAIL — AXI mode currently exits 1 on 401

- [ ] **Step 3: Implement AXI error handling in main.rs**

In `updown/src/main.rs`, modify `main()` and `run()` to catch `ApiError` in AXI mode:

```rust
use updown_lib::client::ApiError;

fn main() {
    let cli = Cli::parse();
    let mode = output::OutputMode::from_flags(cli.json, cli.table);

    if let Err(e) = run(cli) {
        // In AXI mode, known API errors go to stdout with exit 0
        if mode == output::OutputMode::Axi {
            if let Some(api_err) = e.downcast_ref::<ApiError>() {
                output::print_axi_error(api_err.status_code, &api_err.message);
                return;
            }
        }

        eprintln!("Error: {:#}", e);

        let exit_code = if format!("{:#}", e).contains("No API key found")
            || format!("{:#}", e).contains("Failed to parse config")
            || format!("{:#}", e).contains("HOME environment variable")
        {
            2
        } else {
            1
        };
        process::exit(exit_code);
    }
}
```

Note: `mode` must be computed before `run()` to be available in the error path. This means computing it from the raw CLI flags before the full config resolution. Refactor slightly:

```rust
fn main() {
    let cli = Cli::parse();
    let mode = output::OutputMode::from_flags(cli.json, cli.table);

    if let Err(e) = run(cli, mode) {
        if mode == output::OutputMode::Axi {
            if let Some(api_err) = e.downcast_ref::<ApiError>() {
                output::print_axi_error(api_err.status_code, &api_err.message);
                return;
            }
        }

        eprintln!("Error: {:#}", e);

        let exit_code = if format!("{:#}", e).contains("No API key found")
            || format!("{:#}", e).contains("Failed to parse config")
            || format!("{:#}", e).contains("HOME environment variable")
        {
            2
        } else {
            1
        };
        process::exit(exit_code);
    }
}

fn run(cli: Cli, mode: output::OutputMode) -> Result<()> {
    let config = Config::load(cli.api_key.as_deref())?;
    let client = Client::new(config.api_key, config.base_url)?;

    match cli.command {
        Commands::Checks { action } => cmd::checks::run(action, &client, mode),
        Commands::Nodes { action } => cmd::nodes::run(action, &client, mode),
        Commands::Recipients { action } => cmd::recipients::run(action, &client, mode),
        Commands::StatusPages { action } => cmd::status_pages::run(action, &client, mode),
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p updown test_checks_list_axi_error_401 test_checks_list_json_error_401`
Expected: Both PASS

- [ ] **Step 5: Run all tests**

Run: `cargo test --workspace`
Expected: All pass

- [ ] **Step 6: Commit**

```bash
git add updown/src/main.rs updown/tests/checks_test.rs
git commit -m "feat: structured AXI error handling for known API errors"
```

---

### Task 10: Implement content truncation for AXI detail views

**Files:**
- Modify: `updown/src/cmd/checks.rs`

- [ ] **Step 1: Write test — truncation applies to http_body in AXI detail**

Add to `updown/tests/checks_test.rs`:

```rust
#[test]
fn test_checks_get_axi_truncation() {
    let long_body = "x".repeat(300);
    let fixture = format!(
        r#"{{"token":"abc123","url":"https://example.com","http_body":"{}","down":false}}"#,
        long_body
    );

    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks/abc123")
        .match_header("X-API-KEY", "test-key")
        .with_body(&fixture)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "checks", "get", "abc123"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("...[300]"));

    mock.assert();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p updown test_checks_get_axi_truncation`
Expected: FAIL — full 300-char string appears without truncation

- [ ] **Step 3: Apply truncation to Check before AXI detail serialization**

In `updown/src/cmd/checks.rs`, in the `get()` function's `Axi` arm, add truncation before serializing:

```rust
OutputMode::Axi => {
    let mut check: Check = resp.json()?;
    // Truncate long fields for AXI output
    if let Some(ref body) = check.http_body {
        check.http_body = Some(output::truncate_field(body, 200));
    }
    if let Some(ref hdrs) = check.custom_headers {
        let hdrs_str = hdrs.to_string();
        if hdrs_str.len() > 200 {
            check.custom_headers = Some(serde_json::Value::String(
                output::truncate_field(&hdrs_str, 200),
            ));
        }
    }
    let h1 = format!("checks update {}", token);
    let h2 = format!("checks downtimes {}", token);
    let h3 = format!("checks metrics {}", token);
    output::print_axi_detail(&check, &[&h1, &h2, &h3]);
}
```

- [ ] **Step 4: Run test**

Run: `cargo test -p updown test_checks_get_axi_truncation`
Expected: PASS

- [ ] **Step 5: Apply truncation to Downtime.error in checks downtimes AXI arm**

In `updown/src/cmd/checks.rs`, in the `Downtimes` AXI arm, truncate error fields before building rows:

```rust
OutputMode::Axi => {
    let downtimes: Vec<Downtime> = resp.json()?;
    let total_duration: u64 = downtimes.iter().filter_map(|d| d.duration).sum();
    let aggregate = format!(
        "{} downtimes, total_duration: {}s",
        downtimes.len(),
        total_duration
    );
    let rows: Vec<DowntimeRow> = downtimes
        .iter()
        .map(|d| {
            let mut row = DowntimeRow::from(d);
            row.error = output::truncate_field(&row.error, 200);
            row
        })
        .collect();
    let h1 = format!("checks get {}", token);
    let h2 = format!("checks metrics {}", token);
    output::print_axi_list(&rows, "downtimes", &aggregate, &[&h1, &h2]);
}
```

- [ ] **Step 6: Run all tests and clippy**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: All pass, clean clippy

- [ ] **Step 7: Commit**

```bash
git add updown/src/cmd/checks.rs updown/tests/checks_test.rs
git commit -m "feat: content truncation for AXI detail views"
```

---

## Chunk 6: Update Existing Tests + TOON Validation

### Task 11: Validate TOON output format and add AXI integration tests

**Files:**
- Modify: `updown/tests/checks_test.rs`

Existing tests were already updated to use `--table` in Task 2. This task validates TOON output and adds dedicated AXI tests.

- [ ] **Step 1: Validate TOON output format**

Write a test that captures actual `toon-format` output for a Vec of Row types and verifies structure:

```rust
#[test]
fn test_toon_format_check_rows() {
    use updown_lib::models::check::CheckRow;

    let rows = vec![CheckRow {
        token: "abc".to_string(),
        status: "up".to_string(),
        url: "https://example.com".to_string(),
        uptime: "99.97%".to_string(),
        apdex: "0.50".to_string(),
        period: "60s".to_string(),
    }];

    let toon = toon_format::encode_default(&rows).unwrap();
    // Verify it produces some reasonable output (exact format depends on crate)
    assert!(!toon.is_empty());
    assert!(toon.contains("abc") || toon.contains("example.com"));
    // Print it so we can see the actual format during test runs
    eprintln!("TOON output:\n{}", toon);
}
```

This test is diagnostic — it shows us exactly what `toon-format` produces so we can verify it matches the AXI spec's expected format. If the output differs significantly from the `type[count]{fields}:` header format, a follow-up task may be needed to add a custom wrapper.

- [ ] **Step 2: Run all tests**

Run: `cargo test --workspace`
Expected: All pass

- [ ] **Step 3: Commit**

```bash
git add updown/tests/
git commit -m "test: validate TOON output format and add AXI integration tests"
```

---

### Task 12: Final validation

- [ ] **Step 1: Full CI-equivalent check**

Run: `cargo fmt --all --check && cargo clippy --workspace -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps`
Expected: All pass with no warnings

- [ ] **Step 2: Manual smoke test**

If `UPDOWN_API_KEY` is available:
```bash
cargo run -- checks list           # should show TOON + summary + help
cargo run -- --table checks list   # should show ASCII table
cargo run -- --json checks list    # should show JSON
```

- [ ] **Step 3: Commit any remaining fixes**

If any issues found in smoke testing, fix and commit.
