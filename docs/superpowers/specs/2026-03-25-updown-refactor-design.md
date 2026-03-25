# updown Refactor Design Spec

**Date:** 2026-03-25
**Status:** Draft

## Goal

Rename the project from `updown-io` to `updown`, restructure from a single-crate monolith into a Cargo workspace with library + CLI crates, add live tests, perform security cleanup, and prepare for open-source publishing with a clean git history. The forwardemail project at `~/projects/orangerabbit-io/forwardemail` is the structural template.

## Workspace Layout

```
updown/                          # repo root
├── Cargo.toml                   # workspace: members = ["updown-lib", "updown"], resolver = "2"
├── updown-lib/                  # library crate
│   ├── Cargo.toml               # name = "updown-lib"
│   └── src/
│       ├── lib.rs               # pub mod client, config, models
│       ├── client.rs            # HTTP client with X-API-KEY auth
│       ├── config.rs            # 3-tier config loading + unit tests
│       └── models/
│           ├── mod.rs
│           ├── check.rs
│           ├── node.rs
│           ├── recipient.rs
│           └── status_page.rs
├── updown/                      # CLI binary crate
│   ├── Cargo.toml               # name = "updown", depends on updown-lib
│   ├── src/
│   │   ├── main.rs              # clap CLI parser, dispatch, error handling
│   │   ├── output.rs            # OutputMode enum, print_* functions
│   │   └── cmd/
│   │       ├── mod.rs
│   │       ├── checks.rs
│   │       ├── nodes.rs
│   │       ├── recipients.rs
│   │       └── status_pages.rs
│   └── tests/
│       ├── common/
│       │   └── mod.rs           # fixture() + binary() helpers
│       ├── fixtures/            # JSON response fixtures (synthetic data)
│       │   ├── check_get.json
│       │   ├── checks_list.json
│       │   ├── nodes_list.json
│       │   ├── recipients_list.json
│       │   └── status_pages_list.json
│       ├── checks_test.rs       # mockito integration tests
│       ├── nodes_test.rs
│       ├── recipients_test.rs
│       ├── status_pages_test.rs
│       └── live_test.rs         # gated behind UPDOWN_LIVE_TEST=1
```

## Library Crate: `updown-lib`

Contains all domain logic with no CLI concerns.

### Public API

- `Config` — 3-tier config loading: `--api-key` flag > `UPDOWN_API_KEY` env > `~/.config/updown/config.toml`. Base URL overridable via `UPDOWN_BASE_URL`.
- `Client` — blocking HTTP client with `X-API-KEY` auth header. Methods: `get`, `get_with_params`, `get_json`, `get_json_with_params`, `get_text`, `get_text_with_params`, `post`, `put`, `delete`. Error mapping for 401/403/404/422/429.
- `models::*` — API response structs (`Check`, `Node`, `Recipient`, `StatusPage`, `CheckMetrics`, `Downtime`) and tabled `Row` types with `From` impls.

### Dependencies

- `reqwest 0.12` (blocking, json, gzip)
- `serde` (derive), `serde_json`
- `tabled 0.17` — lives in lib because `Row` types derive `Tabled` here. This is a presentation concern in the library, inherited from the forwardemail pattern for consistency.
- `toml 0.8`
- `anyhow`
- Dev: `serial_test`

### Unit Tests

- Config priority chain tests (5 tests, `#[serial]`)
- Model deserialization + Row conversion tests (existing tests move here)

## CLI Crate: `updown`

Binary name: `updown`. Handles clap parsing, output formatting, and command dispatch.

### Dependencies

- `updown-lib` (path = "../updown-lib")
- `clap 4` (derive)
- `serde_json`
- `tabled`
- Dev: `mockito`, `assert_cmd`, `predicates`, `serial_test`, `uuid = { version = "1", features = ["v4"] }`, `reqwest` (blocking, for `StatusCode` assertions in live tests)

### Cargo.toml Metadata

Both crates include full publish metadata:
- `description`, `license = "MIT OR Apache-2.0"`, `repository`, `homepage`, `keywords`, `categories`
- `readme = "../README.md"` (shared README at workspace root)

### Command Structure

Same as current — `Checks`, `Nodes`, `Recipients`, `StatusPages` subcommands. Global flags: `--json`, `--api-key`. Exit codes: 2 for config/auth errors, 1 for API errors.

### Output Module

`OutputMode` enum with `print_json`, `print_table`, `print_kv`, `print_confirm`, `print_raw` functions. Markdown table style via `tabled::Style::markdown()`.

## Live Tests

File: `updown/tests/live_test.rs`, gated behind `UPDOWN_LIVE_TEST=1`.

### Gating Mechanism

A `require_live()` function checks `UPDOWN_LIVE_TEST=1`. If not set, it calls `std::process::exit(0)` which exits the test binary cleanly — tests report as zero run rather than skipped. This matches the forwardemail pattern.

### Helper Functions

- `run_json(args)` — runs CLI with `--json`, asserts success, parses stdout as JSON
- `run_ok(args)` — runs CLI, asserts success, returns stdout as string

### Cleanup Guards

Each resource type gets a guard struct with a `Drop` impl that deletes the resource. This ensures cleanup even if a test panics mid-lifecycle:

- `CheckGuard { token: String }` — calls `checks delete {token}` on drop
- `RecipientGuard { id: String }` — calls `recipients delete {id}` on drop
- `StatusPageGuard { token: String }` — calls `status-pages delete {token}` on drop

### Test Coverage

- **Checks CRUD lifecycle** (`#[serial]`): Create check with UUID alias → list to confirm → get by token → update alias → delete → confirm deletion
  - URL: a harmless target (e.g., `https://httpbin.org/get`)
  - Alias: `test-{uuid}` to avoid conflicts
  - Uses `CheckGuard` for cleanup on panic
- **Recipients CRUD lifecycle** (`#[serial]`): Create webhook recipient with UUID-based URL → list to confirm → delete → confirm deletion
  - Uses webhook type (not email) to avoid triggering real notifications
  - Uses `RecipientGuard` for cleanup on panic
- **Status Pages CRUD lifecycle** (`#[serial]`): Create status page with UUID name → get → update name → delete → confirm deletion
  - Uses `StatusPageGuard` for cleanup on panic
- **Nodes read-only**: List nodes → verify non-empty. Get IPs → verify response.
- All CRUD tests use `--json` flag and parse stdout as JSON for assertions.

### What NOT to test live

- Metrics/downtimes endpoints (require historical data, read-only but noisy)
- Any operation on existing production checks/recipients/status pages

## Rename: `updown-io` → `updown`

### Files to update

| File | Change |
|------|--------|
| Root `Cargo.toml` | Workspace definition (new file) |
| `updown-lib/Cargo.toml` | `name = "updown-lib"`, repository URL, homepage, `readme = "../README.md"` |
| `updown/Cargo.toml` | `name = "updown"`, repository URL, homepage, `readme = "../README.md"` |
| `flake.nix` | `pname = "updown"`, version, binary path, workspace build |
| `.releaserc.json` | Full rewrite for workspace (see CI/CD section) |
| `package.json` | Confirm structure matches forwardemail (private, devDependencies only) |
| `.gitignore` | Add workspace-aware paths + security patterns |
| `.envrc` | Keep as-is (`use flake`, `source_up`, `dotenv`) |
| `README.md` | Project name, install commands, repo URLs |
| `CLAUDE.md` | Project name, architecture description (two-crate workspace) |
| `tests/common/mod.rs` | `cargo_bin!("updown")` |
| `src/cmd/*.rs` | Doc comments |
| `src/main.rs` | Module doc, clap command name |
| `src/config.rs` | Doc comments |
| `.github/workflows/release.yml` | No structural changes needed |

### Repository URL

All references to `orangerabbit-io/updown-io` → `orangerabbit-io/updown`.

## CI/CD Updates

### `.releaserc.json`

Complete configuration matching forwardemail pattern:

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

### `package.json`

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

### `release.yml`

Same structure as forwardemail — push to main triggers semantic-release.

### `.gitignore`

Updated for workspace awareness and security:

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

## Security Cleanup

1. **Scan for leaked secrets**: grep for API keys, tokens, passwords in all committed files and git history
2. **Verify `.gitignore`**: `.env`, `*.pem`, `*.key`, `target/` all excluded (see updated `.gitignore` above)
3. **Audit fixtures**: confirm all JSON fixtures contain synthetic data, no real account info
4. **Config safety**: verify `config.rs` never logs or displays API keys
5. **Dependency check**: review Cargo.lock for known-vulnerable crate versions

## History Reset

Before open-sourcing, squash all git history into a single clean initial commit using an orphan branch approach:

1. Create orphan branch: `git checkout --orphan clean-main`
2. Stage all files: `git add -A`
3. Commit: `git commit -m "Initial commit: updown CLI for updown.io monitoring API"`
4. Replace main: `git branch -D main && git branch -m main`
5. Force push to new remote (the `orangerabbit-io/updown` repo)

This eliminates any risk of secrets, personal data, or messy history leaking into the public repo.

## Nix Flake

Update `flake.nix` for workspace build:
- `pname = "updown"`
- Point at workspace root, build produces `updown` binary
- `devShells.default` with Rust toolchain + openssl + pkg-config

## Out of Scope

- crates.io publishing (separate step after open-source)
- Cross-platform binary builds CI (separate workflow)
- `cargo-deny` / dependency auditing tooling
