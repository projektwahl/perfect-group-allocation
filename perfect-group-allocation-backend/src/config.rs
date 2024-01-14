use figment::providers::{Env, Format, Toml};
use figment::Figment;
use serde::Deserialize;

use crate::error::AppError;

#[derive(Deserialize, Clone)]
pub struct OpenIdConnectConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub openidconnect: Option<OpenIdConnectConfig>,
}

pub fn get_config() -> Result<Config, AppError> {
    Ok(Figment::new()
        .merge(Toml::file("pga.toml"))
        .merge(Env::prefixed("PGA_"))
        .extract()?)
}
