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

## Publishing

- License: MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE)
- Cargo.toml publish metadata: done (description, license, repository, keywords, categories, readme)
- README.md: done
- crates.io: `cargo login` then `cargo publish` (not yet done)
- Nix flake: `packages.default` via `rustPlatform.buildRustPackage` (uses `cargoLock.lockFile`, no hash needed)

### GitHub
- Repo: orangerabbit-io/updown
- CI: semantic-release on push to main

## Conductor verify

<!-- verify -->
```sh
nix develop -c cargo fmt --all --check
```