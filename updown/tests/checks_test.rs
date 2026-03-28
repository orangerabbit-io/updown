mod common;

use mockito::{Matcher, Server};
use predicates::prelude::*;

#[test]
fn test_checks_list_table() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("checks_list.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "--table", "checks", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("abc123"))
        .stdout(predicate::str::contains("example.com"));

    mock.assert();
}

#[test]
fn test_checks_list_json() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("checks_list.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "--json", "checks", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"token\""));

    mock.assert();
}

#[test]
fn test_checks_get() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks/abc123")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("check_get.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "--table",
        "checks",
        "get",
        "abc123",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("abc123"))
    .stdout(predicate::str::contains("example.com"));

    mock.assert();
}

#[test]
fn test_checks_delete() {
    let mut server = Server::new();
    let mock = server
        .mock("DELETE", "/api/checks/abc123")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"deleted": true}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "--table",
        "checks",
        "delete",
        "abc123",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("Deleted check abc123"));

    mock.assert();
}

#[test]
fn test_checks_auth_error() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks")
        .with_status(401)
        .with_body(r#"{"error": "Invalid API key"}"#)
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "bad-key", "checks", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("error{code,message}:"))
        .stdout(predicate::str::contains("401"));

    mock.assert();
}

#[test]
fn test_checks_create() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/checks")
        .match_header("X-API-KEY", "test-key")
        .match_body(Matcher::AllOf(vec![
            Matcher::PartialJsonString(r#"{"url":"https://new.example.com"}"#.into()),
            Matcher::PartialJsonString(r#"{"period":60}"#.into()),
        ]))
        .with_body(r#"{"token":"new123","url":"https://new.example.com"}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "--table",
        "checks",
        "create",
        "https://new.example.com",
        "--period",
        "60",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("Check created: new123"));

    mock.assert();
}

#[test]
fn test_checks_create_pulse_no_url() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/checks")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"token":"pulse1","url":""}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "checks",
        "create",
        "--type",
        "pulse",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success();

    mock.assert();
}

#[test]
fn test_checks_create_requires_url_for_non_pulse() {
    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "checks", "create"])
        .env("UPDOWN_BASE_URL", "http://localhost:1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL is required"));
}

#[test]
fn test_checks_update() {
    let mut server = Server::new();
    let mock = server
        .mock("PUT", "/api/checks/abc123")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"token":"abc123","url":"https://example.com"}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "--table",
        "checks",
        "update",
        "abc123",
        "--period",
        "300",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("Check updated: abc123"));

    mock.assert();
}

#[test]
fn test_checks_downtimes() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks/abc123/downtimes")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"[{"id":"dt1","error":"timeout","started_at":"2024-01-01T00:00:00Z","ended_at":"2024-01-01T01:00:00Z","duration":3600}]"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "--table",
        "checks",
        "downtimes",
        "abc123",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("dt1"))
    .stdout(predicate::str::contains("timeout"));

    mock.assert();
}

#[test]
fn test_checks_metrics_json() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks/abc123/metrics")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"apdex":0.95,"requests":{"samples":100}}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "--json",
        "checks",
        "metrics",
        "abc123",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("apdex"));

    mock.assert();
}

#[test]
fn test_missing_api_key_exit_code_2() {
    let mut cmd = common::binary();
    cmd.args(["checks", "list"])
        .env_remove("UPDOWN_API_KEY")
        .env("HOME", "/tmp/nonexistent-updown-test")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("No API key found"));
}

#[test]
fn test_checks_get_with_metrics() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks/abc123")
        .match_header("X-API-KEY", "test-key")
        .match_query(Matcher::UrlEncoded("metrics".into(), "true".into()))
        .with_body(common::fixture("check_get.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "--table",
        "checks",
        "get",
        "abc123",
        "--metrics",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("abc123"));

    mock.assert();
}

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
        .success()
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
        .failure()
        .stderr(predicate::str::contains("Authentication failed"));

    mock.assert();
}

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

#[test]
fn test_checks_not_found_error() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/checks/nonexistent")
        .with_status(404)
        .with_body(r#"{"error": "Check not found"}"#)
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "checks", "get", "nonexistent"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("error{code,message}:"))
        .stdout(predicate::str::contains("404"));

    mock.assert();
}

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
