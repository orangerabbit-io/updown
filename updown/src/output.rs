//! Output formatting for CLI results.
//!
//! All command handlers receive an [`OutputMode`] and call the appropriate
//! `print_*` function rather than formatting output themselves. This keeps
//! display logic centralized and makes it straightforward to add new output
//! formats in the future.
//!
//! Use `--json` for machine-readable JSON output, `--table` for explicit
//! human-readable table output, or neither for the default Axi format.

use serde::Serialize;
use tabled::{Table, Tabled};
use toon_format::encode_default;

/// Controls how output is rendered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    /// Token-efficient TOON format with aggregates and help hints (default).
    Axi,
    /// Columnar table output suitable for human reading.
    Table,
    /// Pretty-printed JSON suitable for piping or scripting.
    Json,
}

impl OutputMode {
    /// Determines output mode from CLI flags. No flags = Axi (default).
    pub fn from_flags(json: bool, table: bool) -> Self {
        if json {
            OutputMode::Json
        } else if table {
            OutputMode::Table
        } else {
            OutputMode::Axi
        }
    }
}

/// Prints a JSON value as pretty-printed JSON to stdout.
pub fn print_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    );
}

/// Prints a slice of tabled rows as a formatted ASCII table.
///
/// Prints "No results." when `items` is empty rather than an empty table.
pub fn print_table<T: Tabled>(items: &[T]) {
    if items.is_empty() {
        println!("No results.");
        return;
    }
    let table = Table::new(items).to_string();
    println!("{}", table);
}

/// Prints key-value pairs in aligned `key:  value` format.
///
/// All keys are right-aligned to the width of the longest key, producing
/// a visually consistent layout for single-item detail views.
pub fn print_kv(pairs: &[(&str, String)]) {
    let max_key_len = pairs.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in pairs {
        println!("{:>width$}:  {}", key, value, width = max_key_len);
    }
}

/// Prints a confirmation message for mutating operations (create, update, delete).
pub fn print_confirm(message: &str) {
    println!("{}", message);
}

/// Prints text verbatim to stdout without appending a newline.
///
/// Used for passthrough of pre-formatted API responses such as the plain-text
/// IP list from `GET /api/nodes/ips?format=txt`.
pub fn print_raw(text: &str) {
    print!("{}", text);
}

/// Truncates a string field to `max` bytes for AXI output.
/// Fields exceeding `max` are cut and appended with `...[{original_len}]`.
#[allow(dead_code)]
pub fn truncate_field(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    let total = value.len();
    let suffix = format!("...[{}]", total);
    if suffix.len() >= max {
        // Max is too small to fit the suffix — hard-truncate
        let safe = value
            .char_indices()
            .take_while(|(i, c)| i + c.len_utf8() <= max)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        return value[..safe].to_string();
    }
    let keep = max - suffix.len();
    // Find a safe UTF-8 boundary at or before `keep`
    let safe_keep = value
        .char_indices()
        .take_while(|(i, _)| *i <= keep)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}{}", &value[..safe_keep], suffix)
}

/// Formats a list of items as AXI output: summary + TOON + help hints.
#[allow(dead_code)]
pub fn format_axi_list<T: Serialize>(
    items: &[T],
    resource_name: &str,
    aggregate: &str,
    hints: &[&str],
) -> String {
    let mut out = format!("summary: {}\n", aggregate);

    if items.is_empty() {
        out.push_str(&format!("{}[0]:\n", resource_name));
    } else {
        match encode_default(&items) {
            Ok(toon) => out.push_str(&toon),
            Err(_) => out.push_str(&format!(
                "{}[{}]: <encoding error>\n",
                resource_name,
                items.len()
            )),
        }
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }

    if !hints.is_empty() {
        out.push_str(&format!("help[{}]", hints.join(", ")));
    }
    out
}

/// Formats a single item detail as AXI output: TOON + help hints.
#[allow(dead_code)]
pub fn format_axi_detail<T: Serialize>(item: &T, hints: &[&str]) -> String {
    let mut out = match encode_default(&item) {
        Ok(toon) => toon,
        Err(_) => "<encoding error>".to_string(),
    };
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !hints.is_empty() {
        out.push_str(&format!("help[{}]", hints.join(", ")));
    }
    out
}

/// Formats a confirmation message as AXI output with help hints.
#[allow(dead_code)]
pub fn format_axi_confirm(message: &str, hints: &[&str]) -> String {
    let mut out = format!("{}\n", message);
    if !hints.is_empty() {
        out.push_str(&format!("help[{}]", hints.join(", ")));
    }
    out
}

/// Formats a structured error as AXI output (printed to stdout with exit 0).
#[allow(dead_code)]
pub fn format_axi_error(code: u16, message: &str) -> String {
    format!("error{{code,message}}:\n  {},{}\n", code, message)
}

/// Prints AXI list output to stdout.
#[allow(dead_code)]
pub fn print_axi_list<T: Serialize>(
    items: &[T],
    resource_name: &str,
    aggregate: &str,
    hints: &[&str],
) {
    print!("{}", format_axi_list(items, resource_name, aggregate, hints));
}

/// Prints AXI detail output to stdout.
#[allow(dead_code)]
pub fn print_axi_detail<T: Serialize>(item: &T, hints: &[&str]) {
    print!("{}", format_axi_detail(item, hints));
}

/// Prints AXI confirmation to stdout.
#[allow(dead_code)]
pub fn print_axi_confirm(message: &str, hints: &[&str]) {
    print!("{}", format_axi_confirm(message, hints));
}

/// Prints AXI error to stdout.
#[allow(dead_code)]
pub fn print_axi_error(code: u16, message: &str) {
    print!("{}", format_axi_error(code, message));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_mode_default_is_axi() {
        assert_eq!(OutputMode::from_flags(false, false), OutputMode::Axi);
    }

    #[test]
    fn test_output_mode_json_flag() {
        assert_eq!(OutputMode::from_flags(true, false), OutputMode::Json);
    }

    #[test]
    fn test_output_mode_table_flag() {
        assert_eq!(OutputMode::from_flags(false, true), OutputMode::Table);
    }

    #[test]
    fn test_truncate_field_short() {
        assert_eq!(truncate_field("hello", 200), "hello");
    }

    #[test]
    fn test_truncate_field_exact() {
        let s = "a".repeat(200);
        assert_eq!(truncate_field(&s, 200), s);
    }

    #[test]
    fn test_truncate_field_long() {
        let s = "a".repeat(300);
        let result = truncate_field(&s, 200);
        assert!(result.len() < 300);
        assert!(result.ends_with("[300]"));
        assert!(result.contains("..."));
    }

    #[test]
    fn test_truncate_field_tiny_max() {
        let result = truncate_field("hello world", 5);
        assert_eq!(result.len(), 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_axi_list_format() {
        let rows = vec![
            serde_json::json!({"token": "abc", "status": "up"}),
            serde_json::json!({"token": "def", "status": "down"}),
        ];
        let output =
            format_axi_list(&rows, "checks", "2 checks, 1 down", &["checks get <token>"]);
        assert!(output.contains("summary: 2 checks, 1 down"));
        assert!(output.contains("help[checks get <token>]"));
    }

    #[test]
    fn test_format_axi_list_empty() {
        let rows: Vec<serde_json::Value> = vec![];
        let output =
            format_axi_list(&rows, "checks", "0 checks", &["checks create --url <url>"]);
        assert!(output.contains("summary: 0 checks"));
        assert!(output.contains("checks[0]"));
        assert!(output.contains("help[checks create --url <url>]"));
    }

    #[test]
    fn test_format_axi_error() {
        let output = format_axi_error(401, "Authentication failed.");
        assert!(output.contains("error{code,message}:"));
        assert!(output.contains("401,Authentication failed."));
    }

    #[test]
    fn test_format_axi_confirm() {
        let output =
            format_axi_confirm("Check created: abc1", &["checks get abc1", "checks list"]);
        assert!(output.contains("Check created: abc1"));
        assert!(output.contains("help[checks get abc1, checks list]"));
    }

    #[test]
    fn test_format_axi_detail() {
        let item = serde_json::json!({"token": "abc", "url": "https://example.com"});
        let output = format_axi_detail(&item, &["checks update abc"]);
        assert!(output.contains("help[checks update abc]"));
    }
}
