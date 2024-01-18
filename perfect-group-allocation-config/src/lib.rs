use core::fmt::{Debug, Display};

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct OpenIdConnectConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub url: String,
    pub database_url: String,
    pub openidconnect: OpenIdConnectConfig,
}

#[derive(thiserror::Error)]
pub enum ConfigError {
    #[error("config error: {0}")]
    Header(#[from] figment::Error),
}

impl Debug for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

pub fn get_config() -> Result<Config, ConfigError> {
    Ok(Figment::new()
        .merge(Toml::file("pga.toml"))
        .merge(Env::prefixed("PGA_"))
        .extract()?)
}
