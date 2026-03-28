mod common;

use mockito::{Matcher, Server};
use predicates::prelude::*;

#[test]
fn test_nodes_list_table() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/nodes")
        .match_header("X-API-KEY", "test-key")
        .with_body(common::fixture("nodes_list.json"))
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "--table", "nodes", "list"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("lan"))
        .stdout(predicate::str::contains("Los Angeles"));

    mock.assert();
}

#[test]
fn test_nodes_ips_ipv4() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/nodes/ipv4")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"["1.2.3.4", "5.6.7.8"]"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "--table", "nodes", "ips", "--ipv4"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("1.2.3.4"));

    mock.assert();
}

#[test]
fn test_nodes_ips_format_txt() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/nodes/ips")
        .match_query(Matcher::UrlEncoded("format".into(), "txt".into()))
        .match_header("X-API-KEY", "test-key")
        .with_body("1.2.3.4\n5.6.7.8\n")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "nodes", "ips", "--format", "txt"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("1.2.3.4\n5.6.7.8"));

    mock.assert();
}

#[test]
fn test_nodes_ips_ipv4_ipv6_conflict() {
    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "nodes", "ips", "--ipv4", "--ipv6"])
        .env("UPDOWN_BASE_URL", "http://localhost:1")
        .assert()
        .failure();
}

#[test]
fn test_nodes_ips_ipv6() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/nodes/ipv6")
        .match_header("X-API-KEY", "test-key")
        .with_body(r#"["::1", "2001:db8::1"]"#)
        .with_header("content-type", "application/json")
        .create();

    let mut cmd = common::binary();
    cmd.args(["--api-key", "test-key", "--table", "nodes", "ips", "--ipv6"])
        .env("UPDOWN_BASE_URL", server.url())
        .assert()
        .success()
        .stdout(predicate::str::contains("::1"));

    mock.assert();
}
