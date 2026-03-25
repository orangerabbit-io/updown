//! Configuration loading for the updown CLI.
//!
//! The API key is resolved from three sources in priority order:
//! CLI flag → `UPDOWN_API_KEY` env var → `~/.config/updown/config.toml`.
//! The base URL defaults to `https://updown.io` and can be overridden with
//! `UPDOWN_BASE_URL` (primarily for testing against a local server).

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Runtime configuration derived from the CLI flag, environment, or config file.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// updown.io read-write API key.
    pub api_key: String,
    /// Base URL for all API requests. Defaults to `https://updown.io`.
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_base_url() -> String {
    "https://updown.io".to_string()
}

impl Config {
    /// Loads configuration using the three-source priority chain.
    ///
    /// `api_key_override` is the value of the `--api-key` CLI flag, if provided.
    /// When it is `Some`, the config file and environment variable are skipped entirely.
    pub fn load(api_key_override: Option<&str>) -> Result<Self> {
        let base_url = std::env::var("UPDOWN_BASE_URL").unwrap_or_else(|_| default_base_url());

        // 1. CLI flag override
        if let Some(key) = api_key_override {
            return Ok(Config {
                api_key: key.to_string(),
                base_url,
            });
        }

        // 2. Environment variable
        if let Ok(key) = std::env::var("UPDOWN_API_KEY") {
            return Ok(Config {
                api_key: key,
                base_url,
            });
        }

        // 3. Config file
        let path = Self::config_path()?;
        let contents = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "No API key found. Create a config file at {} with:\n\n  api_key = \"your-api-key\"\n\nOr set UPDOWN_API_KEY environment variable.",
                path.display()
            )
        })?;

        let mut config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file at {}", path.display()))?;

        if std::env::var("UPDOWN_BASE_URL").is_ok() {
            config.base_url = base_url;
        }

        Ok(config)
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".config/updown/config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_cli_flag_override_takes_priority() {
        let config = Config::load(Some("flag-key")).unwrap();
        assert_eq!(config.api_key, "flag-key");
    }

    #[test]
    #[serial]
    fn test_env_var_override() {
        std::env::set_var("UPDOWN_API_KEY", "env-key");
        let config = Config::load(None).unwrap();
        assert_eq!(config.api_key, "env-key");
        std::env::remove_var("UPDOWN_API_KEY");
    }

    #[test]
    #[serial]
    fn test_base_url_env_override() {
        std::env::set_var("UPDOWN_BASE_URL", "http://localhost:9999");
        let config = Config::load(Some("key")).unwrap();
        assert_eq!(config.base_url, "http://localhost:9999");
        std::env::remove_var("UPDOWN_BASE_URL");
    }

    #[test]
    #[serial]
    fn test_default_base_url() {
        std::env::remove_var("UPDOWN_BASE_URL");
        let config = Config::load(Some("key")).unwrap();
        assert_eq!(config.base_url, "https://updown.io");
    }

    #[test]
    #[serial]
    fn test_missing_api_key_errors() {
        std::env::remove_var("UPDOWN_API_KEY");
        let result = Config::load(None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No API key found") || err.contains("config"));
    }
}
