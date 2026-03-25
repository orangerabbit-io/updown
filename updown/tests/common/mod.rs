use std::path::PathBuf;

#[allow(dead_code)]
pub fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Missing fixture: {}", path.display()))
}

pub fn binary() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("updown").unwrap()
}
