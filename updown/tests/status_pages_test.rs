mod common;

use mockito::Server;
use predicates::prelude::*;

#[test]
fn test_status_pages_list() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/status_pages")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("status_pages_list.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "status-pages", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("sp123"))
        .stdout(predicate::str::contains("My Status"));

    mock.assert();
}

#[test]
fn test_status_pages_delete() {
    let mut server = Server::new();
    let mock = server
        .mock("DELETE", "/api/status_pages/sp123")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"deleted": true}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "status-pages", "delete", "sp123"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted status page sp123"));

    mock.assert();
}

#[test]
fn test_status_pages_create() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/status_pages")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"token":"spnew","url":"https://status.new.com","name":"New Status"}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "status-pages",
        "create",
        "--checks",
        "abc123,def456",
        "--name",
        "New Status",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("Status page created: spnew"));

    mock.assert();
}

#[test]
fn test_status_pages_update() {
    let mut server = Server::new();
    let mock = server
        .mock("PUT", "/api/status_pages/sp123")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"token":"sp123","url":"https://status.example.com","name":"Updated"}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "status-pages",
        "update",
        "sp123",
        "--name",
        "Updated",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("Status page updated: sp123"));

    mock.assert();
}
