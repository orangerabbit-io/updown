//! Data models for updown.io API resources.
//!
//! Each submodule defines two structs: a full API model that deserializes from JSON,
//! and a flattened row type that implements [`tabled::Tabled`] for table output.

pub mod check;
pub mod node;
pub mod recipient;
pub mod status_page;
