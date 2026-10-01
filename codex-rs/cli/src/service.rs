//! Resolve ccodex's service before any credentials or network clients are loaded.

use codex_http_client::service_endpoint::DEFAULT_BASE_OAUTH_URL;
use codex_http_client::service_endpoint::initialize_service;
use codex_utils_cli::CliConfigOverrides;
use std::path::Path;

pub(crate) fn initialize(config_home: &Path, overrides: &CliConfigOverrides) -> anyhow::Result<()> {
    let path = config_home.join("config.toml");
    let config = match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str::<toml::Value>(&contents)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            toml::Value::Table(Default::default())
        }
        Err(error) => return Err(error.into()),
    };
    let mut value = config.get("BASE_OAUTH_URL").cloned();
    for (key, override_value) in overrides.parse_overrides().map_err(anyhow::Error::msg)? {
        if key == "BASE_OAUTH_URL" {
            value = Some(override_value);
        }
    }
    let value = match value.as_ref() {
        Some(value) => value
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("BASE_OAUTH_URL must be a string"))?,
        None => DEFAULT_BASE_OAUTH_URL,
    };
    initialize_service(value)?;
    Ok(())
}
