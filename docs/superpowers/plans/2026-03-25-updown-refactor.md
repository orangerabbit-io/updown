# updown Refactor Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename from `updown-io` to `updown`, restructure into a Cargo workspace (library + CLI), add live tests, perform security cleanup, and reset history for open-source publishing.

**Architecture:** Two-crate Cargo workspace matching the forwardemail project pattern. `updown-lib` contains the HTTP client, config loading, and API models. `updown` (CLI binary) depends on the library and adds clap parsing, output formatting, and command dispatch.

**Tech Stack:** Rust (blocking reqwest, clap 4, tabled, serde, mockito), Nix flake, semantic-release

**Spec:** `docs/superpowers/specs/2026-03-25-updown-refactor-design.md`

---

## Chunk 1: Workspace Scaffolding and Library Crate

### Task 1: Create workspace root Cargo.toml

**Files:**
- Modify: `Cargo.toml` (replace single-crate config with workspace definition)

- [ ] **Step 1: Replace root Cargo.toml with workspace definition**

```toml
[workspace]
members = ["updown-lib", "updown"]
resolver = "2"
```

- [ ] **Step 2: Create updown-lib directory structure**

```bash
mkdir -p updown-lib/src/models
```

- [ ] **Step 3: Create updown-lib/Cargo.toml**

```toml
[package]
name = "updown-lib"
version = "0.1.0"
edition = "2021"
description = "Rust client library for the updown.io monitoring API"
license = "MIT OR Apache-2.0"
repository = "https://github.com/orangerabbit-io/updown"
homepage = "https://github.com/orangerabbit-io/updown"
readme = "../README.md"
keywords = ["updown", "monitoring", "api", "client"]
categories = ["api-bindings"]

[dependencies]
anyhow = "1"
reqwest = { version = "0.12", features = ["blocking", "json", "gzip"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tabled = "0.17"
toml = "0.8"

[dev-dependencies]
serial_test = "3"
```

- [ ] **Step 4: Create updown-lib/src/lib.rs**

```rust
//! Client library for the [updown.io](https://updown.io) monitoring API.
//!
//! Provides an authenticated HTTP client, configuration loading, and
//! strongly-typed models for all API resources.

pub mod client;
pub mod config;
pub mod models;
```

- [ ] **Step 5: Move source files into library crate**

Copy (do not delete originals yet) the following files:
- `src/client.rs` → `updown-lib/src/client.rs` (no changes needed — no `crate::` imports)
- `src/config.rs` → `updown-lib/src/config.rs` (update doc comment: `updown-io CLI` → `updown CLI`)
- `src/models/mod.rs` → `updown-lib/src/models/mod.rs` (no changes needed)
- `src/models/check.rs` → `updown-lib/src/models/check.rs` (no changes needed)
- `src/models/node.rs` → `updown-lib/src/models/node.rs` (no changes needed)
- `src/models/recipient.rs` → `updown-lib/src/models/recipient.rs` (no changes needed)
- `src/models/status_page.rs` → `updown-lib/src/models/status_page.rs` (no changes needed)

In `updown-lib/src/config.rs`, update line 1 doc comment:

```rust
//! Configuration loading for the updown CLI.
```

- [ ] **Step 6: Verify library crate compiles**

Run: `cargo check -p updown-lib`
Expected: compiles with no errors

- [ ] **Step 7: Run library unit tests**

Run: `cargo test -p updown-lib`
Expected: 12 unit tests pass (5 config + 4 check + 1 node + 1 recipient + 1 status_page)

- [ ] **Step 8: Commit**

```bash
git add updown-lib/ Cargo.toml
git commit -m "refactor: extract updown-lib library crate from monolith"
```

---

### Task 2: Create CLI crate

**Files:**
- Create: `updown/Cargo.toml`
- Create: `updown/src/main.rs`
- Create: `updown/src/output.rs`
- Create: `updown/src/cmd/mod.rs`
- Create: `updown/src/cmd/checks.rs`
- Create: `updown/src/cmd/nodes.rs`
- Create: `updown/src/cmd/recipients.rs`
- Create: `updown/src/cmd/status_pages.rs`

- [ ] **Step 1: Create CLI directory structure**

```bash
mkdir -p updown/src/cmd
mkdir -p updown/tests/common
mkdir -p updown/tests/fixtures
```

- [ ] **Step 2: Create updown/Cargo.toml**

```toml
[package]
name = "updown"
version = "0.1.0"
edition = "2021"
description = "Command-line interface for the updown.io monitoring API"
license = "MIT OR Apache-2.0"
repository = "https://github.com/orangerabbit-io/updown"
homepage = "https://github.com/orangerabbit-io/updown"
readme = "../README.md"
keywords = ["updown", "monitoring", "cli", "uptime"]
categories = ["command-line-utilities"]

[dependencies]
updown-lib = { path = "../updown-lib" }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde_json = "1"
tabled = "0.17"

[dev-dependencies]
mockito = "1"
assert_cmd = "2"
predicates = "3"
serial_test = "3"
uuid = { version = "1", features = ["v4"] }
reqwest = { version = "0.12", features = ["blocking"] }
```

- [ ] **Step 3: Create updown/src/main.rs**

Copy `src/main.rs` to `updown/src/main.rs` with these changes:
- Remove `mod client;`, `mod config;`, `mod models;` declarations (these are now in the library)
- Keep `mod cmd;` and `mod output;` (these stay in the CLI crate)
- Update imports: `use updown_lib::client::Client;` and `use updown_lib::config::Config;`
- Change `#[command(name = "updown-io"` → `#[command(name = "updown"`
- Update doc comment: `updown-io` → `updown`

```rust
//! # updown
//!
//! Command-line interface for the [updown.io](https://updown.io) website monitoring API.
//!
//! Supports full CRUD for checks, recipients, and status pages, plus read access to
//! monitoring nodes and downtime history. Output defaults to a human-readable table;
//! pass `--json` for machine-readable output.
//!
//! ## Configuration
//!
//! The API key is resolved in this order:
//! 1. `--api-key` flag
//! 2. `UPDOWN_API_KEY` environment variable
//! 3. `~/.config/updown/config.toml`

mod cmd;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process;

use updown_lib::client::Client;
use updown_lib::config::Config;

/// Top-level CLI entry point parsed by clap.
#[derive(Parser)]
#[command(name = "updown", about = "CLI for the updown.io monitoring API")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Force JSON output
    #[arg(long, global = true)]
    pub json: bool,

    /// API key (overrides config file and env var)
    #[arg(long, global = true)]
    pub api_key: Option<String>,
}

/// Top-level subcommands, each mapping to an updown.io API resource.
#[derive(Subcommand)]
pub enum Commands {
    /// Manage monitoring checks
    Checks {
        #[command(subcommand)]
        action: cmd::checks::ChecksAction,
    },
    /// View monitoring node locations and IPs
    Nodes {
        #[command(subcommand)]
        action: cmd::nodes::NodesAction,
    },
    /// Manage alert recipients
    Recipients {
        #[command(subcommand)]
        action: cmd::recipients::RecipientsAction,
    },
    /// Manage public status pages
    #[command(name = "status-pages")]
    StatusPages {
        #[command(subcommand)]
        action: cmd::status_pages::StatusPagesAction,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
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

fn run(cli: Cli) -> Result<()> {
    let config = Config::load(cli.api_key.as_deref())?;
    let client = Client::new(config.api_key, config.base_url)?;
    let mode = output::OutputMode::from_json_flag(cli.json);

    match cli.command {
        Commands::Checks { action } => cmd::checks::run(action, &client, mode),
        Commands::Nodes { action } => cmd::nodes::run(action, &client, mode),
        Commands::Recipients { action } => cmd::recipients::run(action, &client, mode),
        Commands::StatusPages { action } => cmd::status_pages::run(action, &client, mode),
    }
}
```

- [ ] **Step 4: Copy output.rs to CLI crate**

Copy `src/output.rs` → `updown/src/output.rs` (no changes needed — no `crate::` imports)

- [ ] **Step 5: Copy cmd/ to CLI crate with updated imports**

Copy each cmd file, updating imports from `crate::` to `updown_lib::`:

For `updown/src/cmd/mod.rs` — copy as-is (no changes needed).

For `updown/src/cmd/checks.rs` — change:
```rust
use crate::client::Client;
use crate::models::check::{Check, CheckRow, Downtime, DowntimeRow};
use crate::output::{self, OutputMode};
```
to:
```rust
use updown_lib::client::Client;
use updown_lib::models::check::{Check, CheckRow, Downtime, DowntimeRow};
use crate::output::{self, OutputMode};
```

Also update the doc comment on the `ChecksAction` enum: `updown-io checks` → `updown checks` (line 10 of the original file).

For `updown/src/cmd/nodes.rs` — change:
```rust
use crate::client::Client;
use crate::models::node::{Node, NodeRow};
use crate::output::{self, OutputMode};
```
to:
```rust
use updown_lib::client::Client;
use updown_lib::models::node::{Node, NodeRow};
use crate::output::{self, OutputMode};
```

Note: The module-level doc comment in `nodes.rs` references `updown.io` (the service name), not `updown-io` (the old binary name) — no change needed. Same applies to the `NodesAction` enum doc comment (`updown-io nodes` → `updown nodes` on line 11).

For `updown/src/cmd/recipients.rs` — change:
```rust
use crate::client::Client;
use crate::models::recipient::{Recipient, RecipientRow};
use crate::output::{self, OutputMode};
```
to:
```rust
use updown_lib::client::Client;
use updown_lib::models::recipient::{Recipient, RecipientRow};
use crate::output::{self, OutputMode};
```

Update the `RecipientsAction` enum doc comment: `updown-io recipients` → `updown recipients` (line 13).

For `updown/src/cmd/status_pages.rs` — change:
```rust
use crate::client::Client;
use crate::models::status_page::{StatusPage, StatusPageRow};
use crate::output::{self, OutputMode};
```
to:
```rust
use updown_lib::client::Client;
use updown_lib::models::status_page::{StatusPage, StatusPageRow};
use crate::output::{self, OutputMode};
```

Update the `StatusPagesAction` enum doc comment: `updown-io status-pages` → `updown status-pages` (line 13).

- [ ] **Step 6: Verify CLI crate compiles**

Run: `cargo check -p updown`
Expected: compiles with no errors

- [ ] **Step 7: Commit**

```bash
git add updown/
git commit -m "refactor: create updown CLI crate with library dependency"
```

---

### Task 3: Migrate tests and remove old src/

**Files:**
- Move: `tests/` → `updown/tests/`
- Delete: `src/` (old monolith source)

- [ ] **Step 1: Copy test files to CLI crate**

```bash
cp tests/common/mod.rs updown/tests/common/mod.rs
cp tests/fixtures/*.json updown/tests/fixtures/
cp tests/checks_test.rs updown/tests/
cp tests/nodes_test.rs updown/tests/
cp tests/recipients_test.rs updown/tests/
cp tests/status_pages_test.rs updown/tests/
```

- [ ] **Step 2: Update tests/common/mod.rs**

Change `cargo_bin("updown-io")` → `cargo_bin("updown")`:

```rust
use std::path::PathBuf;

pub fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Missing fixture: {}", path.display()))
}

pub fn binary() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("updown").unwrap()
}
```

- [ ] **Step 3: Run all workspace tests**

Run: `cargo test --workspace`
Expected: All tests pass (12 unit in lib + 27 integration in CLI = 39 total)

- [ ] **Step 4: Delete old monolith source and tests**

```bash
rm -rf src/ tests/
```

- [ ] **Step 5: Verify workspace still compiles and tests pass**

Run: `cargo test --workspace`
Expected: All 39 tests pass

- [ ] **Step 6: Run clippy and fmt**

Run: `cargo clippy --workspace -- -D warnings && cargo fmt --all --check`
Expected: clean

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor: migrate tests to CLI crate and remove monolith source"
```

---

## Chunk 2: Config Files and Documentation

### Task 4: Update .gitignore

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Replace .gitignore contents**

```
/target
updown-lib/target
updown/target
.direnv
.env
*.pem
*.key
node_modules
```

- [ ] **Step 2: Commit**

```bash
git add .gitignore
git commit -m "chore: update .gitignore for workspace layout and security patterns"
```

---

### Task 5: Update CI/CD config files

**Files:**
- Modify: `.releaserc.json`
- Modify: `package.json`

- [ ] **Step 1: Replace .releaserc.json**

```json
{
  "branches": ["main"],
  "plugins": [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    ["@semantic-release/changelog", {
      "changelogFile": "CHANGELOG.md"
    }],
    ["@semantic-release/exec", {
      "prepareCmd": "sed -i 's/^version = .*/version = \"${nextRelease.version}\"/' updown/Cargo.toml updown-lib/Cargo.toml && sed -i 's/version = \"[0-9]*\\.[0-9]*\\.[0-9]*\";/version = \"${nextRelease.version}\";/' flake.nix && cargo generate-lockfile"
    }],
    ["@semantic-release/git", {
      "assets": ["CHANGELOG.md", "Cargo.lock", "flake.nix", "updown/Cargo.toml", "updown-lib/Cargo.toml"],
      "message": "chore(release): ${nextRelease.version}\n\n${nextRelease.notes}"
    }],
    "@semantic-release/github"
  ]
}
```

- [ ] **Step 2: Replace package.json**

```json
{
  "private": true,
  "devDependencies": {
    "semantic-release": "^24",
    "@semantic-release/changelog": "^6",
    "@semantic-release/exec": "^7",
    "@semantic-release/git": "^10",
    "@semantic-release/github": "^11"
  }
}
```

- [ ] **Step 3: Commit**

```bash
git add .releaserc.json package.json
git commit -m "ci: update semantic-release config for workspace layout"
```

---

### Task 6: Update flake.nix

**Files:**
- Modify: `flake.nix`

- [ ] **Step 1: Update flake.nix for workspace**

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "updown";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            pkg-config
            openssl
          ];
        };
      });
}
```

- [ ] **Step 2: Verify nix build** (optional — only if direnv/nix available)

Run: `nix build`
Expected: `./result/bin/updown` exists

- [ ] **Step 3: Commit**

```bash
git add flake.nix
git commit -m "chore: update flake.nix for updown workspace build"
```

---

### Task 7: Update README.md

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace README.md**

```markdown
# updown

A command-line interface for the [updown.io](https://updown.io) monitoring API.

## Install

### From crates.io

```sh
cargo install updown
```

### From source

```sh
git clone https://github.com/orangerabbit-io/updown.git
cd updown
cargo build --release
cp target/release/updown ~/.local/bin/
```

### With Nix

```sh
nix profile install github:orangerabbit-io/updown
```

## Configuration

Create `~/.config/updown/config.toml`:

```toml
api_key = "your-api-key"
```

Get your API key from [updown.io/settings/edit](https://updown.io/settings/edit).

Alternatively, set the `UPDOWN_API_KEY` environment variable or pass `--api-key` on every command.

Priority: `--api-key` flag > `UPDOWN_API_KEY` env > config file.

## Usage

### Checks

```sh
updown checks list
updown checks get <token>
updown checks get <token> --metrics
updown checks create https://example.com --period 60
updown checks create --type pulse --alias "Cron job"
updown checks update <token> --period 300
updown checks delete <token>
updown checks downtimes <token>
updown checks metrics <token> --from 2024-01-01 --group host
```

### Nodes

```sh
updown nodes list
updown nodes ips
updown nodes ips --ipv4
updown nodes ips --ipv6
updown nodes ips --format txt    # one IP per line, for firewall rules
```

### Recipients

```sh
updown recipients list
updown recipients create email alerts@example.com --name "Ops Team"
updown recipients create webhook https://hooks.slack.com/...
updown recipients delete <id>
```

### Status Pages

```sh
updown status-pages list
updown status-pages create --checks tok1,tok2 --name "System Status"
updown status-pages update <token> --visibility public
updown status-pages delete <token>
```

### Output

Table output by default. Add `--json` to any command for JSON:

```sh
updown checks list --json
updown checks list --json | jq '.[].url'
```

## API Coverage

| Resource | Commands |
|----------|----------|
| Checks | list, get, create, update, delete, downtimes, metrics |
| Nodes | list, ips |
| Recipients | list, create, delete |
| Status Pages | list, create, update, delete |

## Development

```sh
cargo test --workspace     # all tests (unit + integration)
cargo clippy --workspace   # lint
cargo fmt --all            # format
cargo doc --workspace --no-deps  # generate docs
```

### Live Tests

Live tests run against the real updown.io API. They create and delete test resources with UUID names to avoid conflicts.

```sh
UPDOWN_LIVE_TEST=1 cargo test --test live_test -- --test-threads=1
```

Requires `UPDOWN_API_KEY` set via env var or config file.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: update README for updown rename and workspace structure"
```

---

### Task 8: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Replace CLAUDE.md**

```markdown
# updown

Rust CLI for the updown.io monitoring API. Full 1:1 endpoint coverage.

## Build & Test

- `cargo build --workspace` — build all crates
- `cargo test --workspace` — run all tests (unit + integration with mockito)
- `cargo clippy --workspace -- -D warnings` — lint (must be clean)
- `cargo fmt --all --check` — format check
- `cargo doc --workspace --no-deps` — build docs (must be warning-free)
- `nix build` — build via flake (output in `./result/bin/updown`)
- `nix profile install .` — install locally via Nix

## Architecture

Two-crate Cargo workspace. Blocking reqwest (no async). Clap derive for CLI.

### Library: `updown-lib`
- `src/config.rs` — 3-tier config: --api-key flag > UPDOWN_API_KEY env > ~/.config/updown/config.toml
- `src/client.rs` — HTTP client with X-API-KEY auth header
- `src/models/` — API response types + tabled Row types

### CLI: `updown`
- `src/main.rs` — Clap CLI parser, dispatch, error handling
- `src/output.rs` — Table (default) or JSON (--json flag) output
- `src/cmd/` — One file per resource (checks, nodes, recipients, status_pages)
- `tests/` — Integration tests using mockito mock server + assert_cmd

## Testing

- Integration tests use UPDOWN_BASE_URL env var to point at mockito server
- Config tests that mutate env vars must use `#[serial]` from serial_test
- Live tests gated behind `UPDOWN_LIVE_TEST=1` — CRUD lifecycle with UUID names
- Run live tests: `UPDOWN_LIVE_TEST=1 cargo test --test live_test -- --test-threads=1`

## Gotchas

- reqwest `gzip` feature MUST be enabled — client sends Accept-Encoding: gzip header, so API responds with gzip. Without the feature, response parsing fails with "expected value at line 1 column 1"
- period and apdex_t use string-based clap value_parser (not numeric) to avoid compile errors, then parse to numeric in handler code
- Constrained params (period, apdex_t, type, http_verb, locations, visibility, recipient types) are validated by clap value_parser whitelists

## Spec & Plan

- Design spec: docs/superpowers/specs/2026-03-25-updown-refactor-design.md
- Implementation plan: docs/superpowers/plans/2026-03-25-updown-refactor.md

## Publishing

- License: MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE)
- Cargo.toml publish metadata: done (description, license, repository, keywords, categories, readme)
- README.md: done
- crates.io: `cargo login` then `cargo publish` (not yet done)
- Nix flake: `packages.default` via `rustPlatform.buildRustPackage` (uses `cargoLock.lockFile`, no hash needed)

### GitHub
- Repo: orangerabbit-io/updown
- CI: semantic-release on push to main
```

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md for workspace architecture"
```

---

## Chunk 3: Live Tests

### Task 9: Write live_test.rs

**Files:**
- Create: `updown/tests/live_test.rs`

- [ ] **Step 1: Write the live test file**

```rust
//! Live integration tests against the updown.io API.
//!
//! Gated behind UPDOWN_LIVE_TEST=1. Requires UPDOWN_API_KEY set
//! (via env var or config file).
//!
//! Run: UPDOWN_LIVE_TEST=1 cargo test --test live_test -- --test-threads=1

mod common;

use serial_test::serial;

/// Skip test if UPDOWN_LIVE_TEST is not set.
fn require_live() {
    if std::env::var("UPDOWN_LIVE_TEST").unwrap_or_default() != "1" {
        eprintln!("Skipping live test (set UPDOWN_LIVE_TEST=1 to enable)");
        std::process::exit(0);
    }
}

/// Run the CLI with `--json` and return parsed JSON from stdout.
fn run_json(args: &[&str]) -> serde_json::Value {
    let mut cmd = common::binary();
    let output = cmd
        .arg("--json")
        .args(args)
        .output()
        .expect("failed to execute binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Command failed: {:?}\nstdout: {}\nstderr: {}",
        args,
        stdout,
        stderr
    );
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON from {:?}: {}\nstdout: {}", args, e, stdout))
}

/// Run the CLI and assert success, returning stdout as string.
fn run_ok(args: &[&str]) -> String {
    let mut cmd = common::binary();
    let output = cmd.args(args).output().expect("failed to execute binary");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Command failed: {:?}\nstdout: {}\nstderr: {}",
        args,
        stdout,
        stderr
    );
    stdout
}

// ---------------------------------------------------------------------------
// Cleanup guards — ensure resources are deleted even on test panic
// ---------------------------------------------------------------------------

struct CheckGuard {
    token: String,
}

impl Drop for CheckGuard {
    fn drop(&mut self) {
        let _ = common::binary()
            .args(["checks", "delete", &self.token])
            .output();
    }
}

struct RecipientGuard {
    id: String,
}

impl Drop for RecipientGuard {
    fn drop(&mut self) {
        let _ = common::binary()
            .args(["recipients", "delete", &self.id])
            .output();
    }
}

struct StatusPageGuard {
    token: String,
}

impl Drop for StatusPageGuard {
    fn drop(&mut self) {
        let _ = common::binary()
            .args(["status-pages", "delete", &self.token])
            .output();
    }
}

// ---------------------------------------------------------------------------
// Read-only tests — no mutations, safe to run against any account
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn live_checks_list() {
    require_live();
    let json = run_json(&["checks", "list"]);
    assert!(json.is_array(), "checks list should return an array");
}

#[test]
#[serial]
fn live_nodes_list() {
    require_live();
    let json = run_json(&["nodes", "list"]);
    assert!(json.is_object(), "nodes list should return a map");
    assert!(!json.as_object().unwrap().is_empty(), "should have at least one node");
}

#[test]
#[serial]
fn live_nodes_ips() {
    require_live();
    let json = run_json(&["nodes", "ips"]);
    assert!(json.is_array(), "ips should return an array");
    assert!(!json.as_array().unwrap().is_empty(), "should have at least one IP");
}

#[test]
#[serial]
fn live_recipients_list() {
    require_live();
    let json = run_json(&["recipients", "list"]);
    assert!(json.is_array(), "recipients list should return an array");
}

#[test]
#[serial]
fn live_status_pages_list() {
    require_live();
    let json = run_json(&["status-pages", "list"]);
    assert!(json.is_array(), "status-pages list should return an array");
}

// ---------------------------------------------------------------------------
// Check lifecycle: create → list → get → update → delete
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn live_check_lifecycle() {
    require_live();
    let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let alias = format!("test-{}", id);

    // Create
    let json = run_json(&[
        "checks",
        "create",
        "https://httpbin.org/get",
        "--alias",
        &alias,
        "--period",
        "3600",
    ]);
    let token = json
        .get("token")
        .and_then(|v| v.as_str())
        .expect("check create should return token")
        .to_string();
    let _guard = CheckGuard { token: token.clone() };

    // List — should contain our check
    let json = run_json(&["checks", "list"]);
    let checks = json.as_array().expect("should be array");
    assert!(
        checks.iter().any(|c| c.get("token").and_then(|v| v.as_str()) == Some(&token)),
        "checks list should contain created check"
    );

    // Get
    let json = run_json(&["checks", "get", &token]);
    assert_eq!(
        json.get("alias").and_then(|v| v.as_str()),
        Some(alias.as_str())
    );

    // Update alias
    let updated_alias = format!("updated-{}", id);
    let stdout = run_ok(&["checks", "update", &token, "--alias", &updated_alias]);
    assert!(stdout.contains("Check updated"), "should confirm update");

    // Verify update
    let json = run_json(&["checks", "get", &token]);
    assert_eq!(
        json.get("alias").and_then(|v| v.as_str()),
        Some(updated_alias.as_str())
    );

    // Delete (guard will also try, but explicit is better for assertion)
    let stdout = run_ok(&["checks", "delete", &token]);
    assert!(stdout.contains("Deleted check"), "should confirm deletion");
}

// ---------------------------------------------------------------------------
// Recipient lifecycle: create → list → delete
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn live_recipient_lifecycle() {
    require_live();
    let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let webhook_url = format!("https://httpbin.org/post?test={}", id);

    // Create webhook recipient (avoids needing a real email/phone)
    let json = run_json(&[
        "recipients",
        "create",
        "webhook",
        &webhook_url,
        "--name",
        &format!("test-{}", id),
    ]);
    let rec_id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("recipient create should return id")
        .to_string();
    let _guard = RecipientGuard { id: rec_id.clone() };

    // List — should contain our recipient
    let json = run_json(&["recipients", "list"]);
    let recipients = json.as_array().expect("should be array");
    assert!(
        recipients.iter().any(|r| r.get("id").and_then(|v| v.as_str()) == Some(&rec_id)),
        "recipients list should contain created recipient"
    );

    // Delete
    let stdout = run_ok(&["recipients", "delete", &rec_id]);
    assert!(stdout.contains("Deleted recipient"), "should confirm deletion");
}

// ---------------------------------------------------------------------------
// Status page lifecycle: create → get (via list) → update → delete
// Note: status pages require at least one check token. We use an existing
// check from the account. If the account has no checks, this test will fail
// at creation time with a validation error.
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn live_status_page_lifecycle() {
    require_live();
    let id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let name = format!("test-{}", id);

    // Get an existing check token to use
    let checks_json = run_json(&["checks", "list"]);
    let checks = checks_json.as_array().expect("should be array");
    if checks.is_empty() {
        eprintln!("Skipping status page lifecycle: no checks in account");
        return;
    }
    let check_token = checks[0]
        .get("token")
        .and_then(|v| v.as_str())
        .expect("check should have token")
        .to_string();

    // Create
    let json = run_json(&[
        "status-pages",
        "create",
        "--checks",
        &check_token,
        "--name",
        &name,
    ]);
    let sp_token = json
        .get("token")
        .and_then(|v| v.as_str())
        .expect("status page create should return token")
        .to_string();
    let _guard = StatusPageGuard { token: sp_token.clone() };

    // List — should contain our status page
    let json = run_json(&["status-pages", "list"]);
    let pages = json.as_array().expect("should be array");
    assert!(
        pages.iter().any(|p| p.get("token").and_then(|v| v.as_str()) == Some(&sp_token)),
        "status pages list should contain created page"
    );

    // Update name
    let updated_name = format!("updated-{}", id);
    let stdout = run_ok(&["status-pages", "update", &sp_token, "--name", &updated_name]);
    assert!(stdout.contains("Status page updated"), "should confirm update");

    // Delete
    let stdout = run_ok(&["status-pages", "delete", &sp_token]);
    assert!(stdout.contains("Deleted status page"), "should confirm deletion");
}
```

- [ ] **Step 2: Verify live tests compile (without running them)**

Run: `cargo test --test live_test --no-run`
Expected: compiles successfully

- [ ] **Step 3: Commit**

```bash
git add updown/tests/live_test.rs
git commit -m "test: add live integration tests with cleanup guards"
```

---

## Chunk 4: Security Cleanup and History Reset

### Task 10: Security audit

**Files:** (read-only audit, no file changes expected)

- [ ] **Step 1: Scan for potential secrets in tracked files**

Run: `git grep -i -E '(api[_-]?key|token|secret|password|credential)' -- ':!*.md' ':!*.json' ':!*.toml' ':!*.lock' ':!*.nix' ':!docs/'`

Review each match — verify none contain actual secrets. All matches should be variable names, parameter names, or test fixtures with synthetic data.

- [ ] **Step 2: Verify .env is not tracked**

Run: `git ls-files .env`
Expected: no output (file is gitignored)

- [ ] **Step 3: Audit fixture data**

Review each file in `updown/tests/fixtures/`:
- `checks_list.json` — synthetic: token "abc123", url "example.com" ✓
- `check_get.json` — synthetic: same test data ✓
- `nodes_list.json` — synthetic: "1.2.3.4", "Los Angeles" ✓
- `recipients_list.json` — synthetic: "rec123", "admin@example.com" ✓
- `status_pages_list.json` — synthetic: "sp123", "status.example.com" ✓

All fixture data uses example.com domains and dummy tokens. No real account data.

- [ ] **Step 4: Verify config.rs doesn't expose API keys**

Check that `config.rs` never uses `println!`, `eprintln!`, `dbg!`, or `log::` with the API key value. The `Config` struct derives `Debug` but the API key would only appear in debug output — this is acceptable for a CLI tool (user's own terminal).

- [ ] **Step 5: Check git history for leaked secrets**

Run: `git log --all --diff-filter=A --name-only --format="" | sort -u | grep -iE '\.env$|secret|credential|\.pem$|\.key$'`
Expected: only `.env` and `.envrc` (both gitignored). If `.env` was ever committed, the history reset in Task 11 eliminates it.

- [ ] **Step 6: Commit security findings (if any fixes needed)**

If issues found, fix and commit. Otherwise, no commit needed for this task.

---

### Task 11: Delete package-lock.json and regenerate

**Files:**
- Delete: `package-lock.json` (will be regenerated by `nix-shell -p nodejs --run 'npm install'`)

The existing `package-lock.json` was generated from the old `package.json` which was missing `semantic-release` and `@semantic-release/github`. After updating `package.json` in Task 5, the lock file is stale.

- [ ] **Step 1: Delete old lockfile and regenerate**

```bash
rm package-lock.json
nix-shell -p nodejs --run 'npm install'
```

- [ ] **Step 2: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore: regenerate package-lock.json with complete semantic-release deps"
```

---

### Task 12: Remove old docs/superpowers content

**Files:**
- Delete: `docs/superpowers/plans/2026-03-15-updown-io-cli.md` (old plan, references updown-io)
- Delete: `docs/superpowers/specs/2026-03-15-updown-io-cli-design.md` (old spec, references updown-io)

These reference the old project name and are no longer needed. The new spec and plan live alongside them.

- [ ] **Step 1: Delete old docs**

```bash
rm docs/superpowers/plans/2026-03-15-updown-io-cli.md
rm docs/superpowers/specs/2026-03-15-updown-io-cli-design.md
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "chore: remove old updown-io spec and plan documents"
```

---

### Task 13: Final verification

- [ ] **Step 1: Full test suite**

Run: `cargo test --workspace`
Expected: All tests pass

- [ ] **Step 2: Clippy clean**

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings

- [ ] **Step 3: Format check**

Run: `cargo fmt --all --check`
Expected: clean

- [ ] **Step 4: Doc build**

Run: `cargo doc --workspace --no-deps`
Expected: no warnings

- [ ] **Step 5: Nix build** (optional)

Run: `nix build`
Expected: `./result/bin/updown` exists and runs

---

### Task 14: History reset

**IMPORTANT: This is a destructive operation. Confirm with user before executing.**

- [ ] **Step 1: Verify all work is committed**

Run: `git status`
Expected: clean working tree

- [ ] **Step 2: Create orphan branch with clean history**

```bash
git checkout --orphan clean-main
git add -A
git commit -m "Initial commit: updown CLI for updown.io monitoring API"
```

- [ ] **Step 3: Replace main branch**

```bash
git branch -D main
git branch -m main
```

- [ ] **Step 4: Verify clean history**

Run: `git log --oneline`
Expected: single commit

- [ ] **Step 5: Verify everything still works**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: all pass

Note: Do NOT force push yet. The user will create the `orangerabbit-io/updown` repo and push when ready.
