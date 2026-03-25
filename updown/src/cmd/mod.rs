//! Command handlers for each updown.io API resource.
//!
//! Each submodule defines a `*Action` enum (the clap subcommands) and a `run()`
//! function that dispatches to the appropriate API call and formats the result.

pub mod checks;
pub mod nodes;
pub mod recipients;
pub mod status_pages;
