//! Output formatting for CLI results.
//!
//! All command handlers receive an [`OutputMode`] and call the appropriate
//! `print_*` function rather than formatting output themselves. This keeps
//! display logic centralized and makes it straightforward to add new output
//! formats in the future.
//!
//! Use `--json` for machine-readable JSON output, `--table` for explicit
//! human-readable table output, or neither for the default Axi format.

use tabled::{Table, Tabled};

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
}
