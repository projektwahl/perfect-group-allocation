use core::fmt::{Debug, Display};
use std::{
    path::{Path, PathBuf},
    result,
    sync::Arc,
};

use notify::{RecursiveMode, Watcher as _};

#[derive(Debug, Default)]
pub struct OpenIdConnectConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Default)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
}

#[derive(Debug, Default)]
pub struct Config {
    pub url: String,
    pub database_url: String,
    pub openidconnect: OpenIdConnectConfig,
    pub tls: TlsConfig,
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

pub async fn reread_config(config_directory: &Path) -> Result<Config, ConfigError> {
    let url = tokio::fs::read_to_string(config_directory.join("url")).await?;
    let database_url = tokio::fs::read_to_string(config_directory.join("database_url")).await?;
    let issuer_url =
        tokio::fs::read_to_string(config_directory.join("openidconnect/issuer_url")).await?;
    let client_id =
        tokio::fs::read_to_string(config_directory.join("openidconnect/client_id")).await?;
    let client_secret =
        tokio::fs::read_to_string(config_directory.join("openidconnect/client_secret")).await?;
    let cert = tokio::fs::read_to_string(config_directory.join("tls/cert")).await?;
    let key = tokio::fs::read_to_string(config_directory.join("tls/key")).await?;

    Ok(Config {
        url,
        database_url,
        openidconnect: OpenIdConnectConfig {
            issuer_url,
            client_id,
            client_secret,
        },
        tls: TlsConfig { cert, key },
    })
}

// TODO FIXME the openid config should also be fetched here so it is consistent with the rest of the config. may be relevant for redirect url or so

/// https://kubernetes.io/docs/concepts/configuration/secret/#using-secrets-as-files-from-a-pod
/// https://kubernetes.io/docs/tasks/inject-data-application/distribute-credentials-secure/#create-a-pod-that-has-access-to-the-secret-data-through-a-volume
/// Secrets can be hot-reloaded so we can update configuration at runtime
pub async fn get_config() -> Result<tokio::sync::watch::Receiver<Arc<Config>>, ConfigError> {
    let config_directory = std::env::var_os("PGA_CONFIG_DIR").unwrap();
    let config_directory = PathBuf::from(config_directory);
    let notify = Arc::new(tokio::sync::Notify::new());
    let notify2 = notify.clone();

    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            println!("event: {:?}", event);
            notify.notify_one();
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;

    watcher.watch(&config_directory, RecursiveMode::Recursive)?;

    let (tx, rx) = tokio::sync::watch::channel(Arc::new(Config::default()));
    let config_directory2 = config_directory.clone();

    // we watched before so it should be safe to first read the initial config and then watch for changes.
    let new_config = reread_config(&config_directory).await?;
    tx.send(Arc::new(new_config)).unwrap();

    tokio::spawn(async move {
        loop {
            notify2.notified().await;
            // TODO FIXME don't unwrap but log
            let new_config = reread_config(&config_directory2).await.unwrap();
            tx.send(Arc::new(new_config)).unwrap();
        }
    });

    Ok(rx)
}
