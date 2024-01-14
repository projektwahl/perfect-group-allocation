use figment::providers::{Env, Format, Json, Toml};
use figment::Figment;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenIdConnectConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub openidconnect: Option<OpenIdConnectConfig>,
}

pub fn get_config() {
    let config: Config = Figment::new()
        .merge(Toml::file("pga.toml"))
        .merge(Env::prefixed("PGA_"))
        .extract()?;
}
