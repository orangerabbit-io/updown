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
