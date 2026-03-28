# AXI Output Mode for updown CLI

## Goal

Add AXI-compliant output as the default mode for the updown CLI, optimizing for AI agent consumption. Implements 7 of the 10 AXI principles (excluding ambient context, which is handled by existing skills).

## Decisions

- **Default output is AXI.** `--json` and `--table` are opt-in for humans.
- **TOON serialization via `toon-format` crate** (81k downloads, serde-compatible). No hand-rolled format.
- **Content truncation at 200 chars**, fixed, no flag.
- **`help[]` blocks in AXI mode only.** Table and JSON output stay clean.
- **Ambient context skipped.** Existing skills handle agent discoverability.

## 1. Output Mode & CLI Changes

`OutputMode` gains a third variant:

```rust
pub enum OutputMode {
    Axi,    // default
    Table,  // --table
    Json,   // --json
}
```

`--json` and `--table` are mutually exclusive optional flags. No flag = `Axi`.

This is a breaking change to the CLI interface (previously default was Table, `--json` was opt-in). Acceptable because the project has one commit and no published release.

## 2. TOON Formatting

### Dependency

`toon-format` added to the `updown` CLI crate only. `updown-lib` remains transport-agnostic.

### Serialization strategy

- **List views:** Serialize the existing Row types (`CheckRow`, `NodeRow`, etc.) which already contain 4-6 fields each — close to AXI's minimal field recommendation. Row types currently derive `Debug, Tabled` only; `Serialize` must be added (serde is already a dependency of `updown-lib`).
- **Detail views** (`checks get`, etc.): Serialize the full API model struct, since the agent asked for details.
- **Mutations** (create/update/delete): Confirmation message or full model in TOON.

### Output functions in `output.rs`

```rust
fn print_axi_list<T: Serialize>(items: &[T], resource_name: &str, aggregate: &str, hints: &[&str])
fn print_axi_detail<T: Serialize>(item: &T, hints: &[&str])
fn print_axi_confirm(message: &str, hints: &[&str])
fn print_axi_error(message: &str, code: u16)
```

## 3. Pre-computed Aggregates

A `summary:` line prepended to list output, computed in the handler from the full API response.

| Command | Aggregate |
|---|---|
| `checks list` | `{total} checks, {down_count} down, worst_uptime: {min}%` |
| `checks downtimes` | `{total} downtimes, total_duration: {sum}s` |
| `checks metrics` | `apdex: {apdex}` (metrics internals are opaque `serde_json::Value`; only surface `apdex` which is a typed field) |
| `nodes list` | `{total} nodes across {country_count} countries` |
| `recipients list` | `{total} recipients ({type_counts})` e.g. `3 email, 1 slack` |
| `status-pages list` | `{total} status pages, {public_count} public, {checks_count} total checks monitored` |

No aggregate for single-item detail views or mutations.

### Example output

```
summary: 3 checks, 1 down, worst_uptime: 98.50%
checks[3]{token,status,url,uptime}:
  abc1,up,https://api.example.com,99.97%
  def2,down,https://web.example.com,98.50%
  ghi3,up,https://cdn.example.com,100.00%
help[checks get <token> --metrics, checks downtimes <token>]
```

## 4. Contextual `help[]` Blocks

Static per-command hints, AXI mode only. After mutations, hints include the concrete token/id from the response.

| Command | help[] |
|---|---|
| `checks list` | `checks get <token> --metrics, checks downtimes <token>` |
| `checks get <token>` | `checks update <token>, checks downtimes <token>, checks metrics <token>` |
| `checks create` | `checks get <token>, checks list` (token from response) |
| `checks update` | `checks get <token>, checks list` |
| `checks delete` | `checks list` |
| `checks downtimes` | `checks get <token>, checks metrics <token>` |
| `checks metrics` | `checks get <token>, checks downtimes <token>` |
| `nodes list` | `nodes ips --ipv4, nodes ips --ipv6` |
| `nodes ips` | `nodes list` |
| `recipients list` | `recipients create <type> <value>` |
| `recipients create` | `recipients list` (id from response) |
| `recipients delete` | `recipients list` |
| `status-pages list` | `status-pages create --checks <tokens>` |
| `status-pages create` | `status-pages list` (token from response) |
| `status-pages update` | `status-pages list` |
| `status-pages delete` | `status-pages list` |

## 5. Structured Errors & Empty States

### Errors

Known API errors (401, 403, 404, 422, 429) in AXI mode are written to stdout with exit code 0:

```
error{code,message}:
  401,Authentication failed. Check your API key.
```

Unexpected errors (network failures, parse errors) still go to stderr with exit code 1.

Exit code 0 for handled API errors is intentional: agents read stdout, not exit codes. Structured error output is more useful for agent task success than unix exit code conventions. Shell scripts using `set -e` should use `--table` or `--json` mode.

**Implementation:** The `Client` methods that return `Response` objects already call `check_status()` which bails on non-2xx. For AXI mode, introduce a new `Client` method `get_json_axi<T>(path) -> Result<T, ApiError>` that returns a typed `ApiError { code: u16, message: String }` instead of bailing via anyhow. Alternatively, the handlers can catch the anyhow error, pattern-match on the message string (e.g., "Authentication failed"), and route to `print_axi_error()`. The first approach is cleaner — add an `ApiError` enum to `updown-lib` that `check_status()` returns, then let the CLI handlers decide how to render it based on output mode.

### Empty states

```
summary: 0 checks
checks[0]{token,status,url,uptime}:
help[checks create --url <url>]
```

The `[0]` count + summary confirming zero + hint toward create. No ambiguity.

## 6. Content Truncation

AXI mode only. `--json` and `--table` return full untruncated data.

### Fields subject to truncation

- `Check.http_body`
- `Check.custom_headers` (serialized JSON)
- `StatusPage.description`
- `Downtime.error`

### Format

`{first 197 chars}...[{N}]` where N is original total length.

### Implementation

A `truncate_field(value: &str, max: usize) -> String` utility in `output.rs`. Applied as a post-processing pass on Row types after conversion, before TOON serialization. One conversion path, optional truncation for AXI mode.

## Files Changed

| File | Change |
|---|---|
| `updown/Cargo.toml` | Add `toon-format` dependency |
| `updown-lib/src/models/*.rs` | Add `Serialize` derive to Row types (`CheckRow`, `NodeRow`, `RecipientRow`, `StatusPageRow`, `DowntimeRow`) |
| `updown-lib/src/client.rs` | Add `ApiError` enum, refactor `check_status()` to return typed errors instead of anyhow strings |
| `updown/src/main.rs` | Replace `--json` bool with `--json`/`--table` mutually exclusive flags, default to `Axi`. Replace `OutputMode::from_json_flag()` usage with new constructor. |
| `updown/src/output.rs` | Add `Axi` variant, replace `from_json_flag()` with `from_flags(json, table)`, add `print_axi_*` functions, `truncate_field` utility |
| `updown/src/cmd/checks.rs` | Add `Axi` branches with aggregates and hints |
| `updown/src/cmd/nodes.rs` | Add `Axi` branches with aggregates and hints |
| `updown/src/cmd/recipients.rs` | Add `Axi` branches with aggregates and hints |
| `updown/src/cmd/status_pages.rs` | Add `Axi` branches with aggregates and hints |
| `updown/tests/` | Update integration tests for new default output, add AXI-specific tests |

## Edge Cases

- **`nodes ips` command:** Returns raw text from the API (IP list). In AXI mode, pass through unchanged — it's already minimal text. No TOON wrapping needed. `help[]` block still appended.
- **`checks metrics` command:** Metrics are opaque nested JSON (`serde_json::Value`). In AXI mode, serialize via `toon-format` which handles `serde_json::Value` natively. The aggregate is limited to `apdex` since the inner structure is untyped.
- **TOON output format validation:** The example output in Section 3 assumes `toon-format` produces `type[count]{fields}:` headers for `Vec<struct>`. This must be validated during implementation — if the crate's actual output differs, a thin wrapper or custom serialization may be needed to produce the AXI-canonical format.
- **`status-pages list` aggregate:** `StatusPage.checks` is `Option<Vec<String>>`. Computing total checks monitored must handle `None` (treat as 0).

## Out of Scope

- Ambient context / session hooks (handled by existing skills)
- TOON deserialization (agents don't send TOON to us)
- Configurable truncation length
- `--axi` flag (it's the default, no flag needed)
