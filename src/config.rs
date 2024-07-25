use std::{ops::Deref, path::Path};

use anyhow::Result;
use async_gigachat::api::{API_BASE_URL, AUTH_URL, SCOPE_CORPORATE, SCOPE_PERSONAL};
use figment::{
    providers::{Format, Json, Serialized, Toml, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};

macro_rules! package_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

macro_rules! local_config_name {
    ($ext:expr) => {
        concat!(package_name!(), $ext)
    };
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub ollama: Option<OllamaConfig>,
    pub giga: Option<GigaChatConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OllamaConfig {
    pub model: String,
    pub api_url: String,
    pub api_port: u16,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GigaChatConfig {
    pub auth_token: Option<String>,
    pub scope: Option<String>,
    pub auth_url: Option<String>,
    pub api_base_url: Option<String>,
}

impl Default for GigaChatConfig {
    fn default() -> Self {
        Self {
            // Exprected Option<String> found String
            auth_token: match std::env::var("GIGACHAT_AUTH_TOKEN") {
                Ok(token) => Some(token),
                Err(_) => None,
            },
            scope: Some(SCOPE_PERSONAL.into()),
            auth_url: Some(AUTH_URL.to_owned()),
            api_base_url: Some(API_BASE_URL.to_owned()),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama: None,
            giga: Some(GigaChatConfig::default()),
        }
    }
}

impl Config {
    pub fn parse<T: AsRef<Path>>(dir: Option<T>) -> Result<Self> {
        dir.map(Self::parse_from_file)
            .unwrap_or_else(Self::parse_from_cfgdir)
    }

    pub fn parse_from_cfgdir() -> Result<Self> {
        let dirs = dirs::config_dir()
            .map(|d| d.join(package_name!()))
            .ok_or_else(|| anyhow::anyhow!("could not resolve project directories"))?;

        Ok(Figment::from(Serialized::defaults(Config::new()))
            .merge(Toml::file(local_config_name!(".toml")))
            .merge(Yaml::file(local_config_name!(".yaml")))
            .merge(Json::file(local_config_name!(".json")))
            .merge(Toml::file(dirs.join("config.toml")))
            .merge(Yaml::file(dirs.join("config.yml")))
            .merge(Json::file(dirs.join("config.json")))
            .extract()?)
    }

    pub fn parse_from_file<T: AsRef<Path>>(path: T) -> Result<Self> {
        let ext = path.as_ref().extension().unwrap_or_default();
        let mut figment = Figment::from(Serialized::defaults(Config::new()));

        figment = match ext.to_string_lossy().deref() {
            "yml" | "yaml" => figment.merge(Yaml::file(path)),
            "toml" => figment.merge(Toml::file(path)),
            "json" => figment.merge(Json::file(path)),
            _ => anyhow::bail!("invalid config file type"),
        };

        Ok(figment.extract()?)
    }
}
