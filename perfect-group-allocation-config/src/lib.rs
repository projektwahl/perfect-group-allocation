use core::fmt::{Debug, Display};
use std::{path::Path, result};

use notify::{RecursiveMode, Watcher as _};

pub struct OpenIdConnectConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
}

pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

pub struct Config {
    pub url: String,
    pub database_url: String,
    pub openidconnect: OpenIdConnectConfig,
}

#[derive(thiserror::Error)]
pub enum ConfigError {
    #[error("notify error {0}")]
    Notify(#[from] notify::Error),
    #[error("io error {0}")]
    Io(#[from] std::io::Error),
}

impl Debug for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

pub async fn reread_config(config_dir: &Path) -> Result<Config, ConfigError> {
    let url = tokio::fs::read_to_string(config_dir.join("url")).await?;

    Ok(todo!())
}

/// https://kubernetes.io/docs/concepts/configuration/secret/#using-secrets-as-files-from-a-pod
/// https://kubernetes.io/docs/tasks/inject-data-application/distribute-credentials-secure/#create-a-pod-that-has-access-to-the-secret-data-through-a-volume
/// Secrets can be hot-reloaded so we can update configuration at runtime
pub fn get_config() -> Result<Config, ConfigError> {
    let config_directory = std::env::var_os("PGA_CONFIG_DIR").unwrap();
    let notify = tokio::sync::Notify::new();

    let config = None;

    let mut watcher = notify::recommended_watcher(|res| match res {
        Ok(event) => {
            println!("event: {:?}", event);
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;

    watcher.watch(&Path::new(&config_directory), RecursiveMode::Recursive)?;
}
