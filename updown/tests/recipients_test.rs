mod common;

use mockito::{Matcher, Server};
use predicates::prelude::*;

#[test]
fn test_recipients_list() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/recipients")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("recipients_list.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "recipients", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("rec123"))
        .stdout(predicate::str::contains("email"));

    mock.assert();
}

#[test]
fn test_recipients_delete() {
    let mut server = Server::new();
    let mock = server
        .mock("DELETE", "/api/recipients/rec123")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"deleted": true}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "recipients", "delete", "rec123"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted recipient rec123"));

    mock.assert();
}

#[test]
fn test_recipients_create() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/recipients")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"{"id":"rec456","type":"email","value":"new@example.com","selected":true}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "recipients",
        "create",
        "email",
        "new@example.com",
        "--name",
        "New Admin",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success()
    .stdout(predicate::str::contains("Recipient created: rec456"));

    mock.assert();
}

#[test]
fn test_recipients_create_no_selected() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/recipients")
        .match_header("X-API-KEY", "test-key")
        .match_body(Matcher::PartialJsonString(r#"{"selected":false}"#.into()))
        .with_body(r#"{"id":"rec789","type":"email","value":"opt@example.com","selected":false}"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args([
        "--api-key",
        "test-key",
        "recipients",
        "create",
        "email",
        "opt@example.com",
        "--no-selected",
    ])
    .env("UPDOWN_BASE_URL", server.url())
    .assert()
    .success();

    mock.assert();
}
