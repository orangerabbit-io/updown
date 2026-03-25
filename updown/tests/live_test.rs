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
    assert!(
        !json.as_object().unwrap().is_empty(),
        "should have at least one node"
    );
}

#[test]
#[serial]
fn live_nodes_ips() {
    require_live();
    let json = run_json(&["nodes", "ips"]);
    assert!(json.is_array(), "ips should return an array");
    assert!(
        !json.as_array().unwrap().is_empty(),
        "should have at least one IP"
    );
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
// Check lifecycle: create → get → update → delete
// Note: list verification skipped — updown.io API has eventual consistency
// on list queries, and list is already tested by live_checks_list.
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
    let _guard = CheckGuard {
        token: token.clone(),
    };

    // Get
    let json = run_json(&["checks", "get", &token]);
    assert_eq!(
        json.get("alias").and_then(|v| v.as_str()),
        Some(alias.as_str())
    );

    // Update alias — verify via the update response (immediate GET may return stale data)
    let updated_alias = format!("updated-{}", id);
    let json = run_json(&["checks", "update", &token, "--alias", &updated_alias]);
    assert_eq!(
        json.get("alias").and_then(|v| v.as_str()),
        Some(updated_alias.as_str()),
        "update response should reflect new alias"
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
        recipients
            .iter()
            .any(|r| r.get("id").and_then(|v| v.as_str()) == Some(&rec_id)),
        "recipients list should contain created recipient"
    );

    // Delete
    let stdout = run_ok(&["recipients", "delete", &rec_id]);
    assert!(
        stdout.contains("Deleted recipient"),
        "should confirm deletion"
    );
}

// ---------------------------------------------------------------------------
// Status page lifecycle: create → list → update → delete
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
    let _guard = StatusPageGuard {
        token: sp_token.clone(),
    };

    // List — should contain our status page
    let json = run_json(&["status-pages", "list"]);
    let pages = json.as_array().expect("should be array");
    assert!(
        pages
            .iter()
            .any(|p| p.get("token").and_then(|v| v.as_str()) == Some(&sp_token)),
        "status pages list should contain created page"
    );

    // Update name
    let updated_name = format!("updated-{}", id);
    let stdout = run_ok(&["status-pages", "update", &sp_token, "--name", &updated_name]);
    assert!(
        stdout.contains("Status page updated"),
        "should confirm update"
    );

    // Delete
    let stdout = run_ok(&["status-pages", "delete", &sp_token]);
    assert!(
        stdout.contains("Deleted status page"),
        "should confirm deletion"
    );
}
