use std::collections::HashMap;
use std::io::{Error, Read};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default = "default_providers")]
    pub notification_providers : std::collections::HashMap<String, serde_yaml::Value>,
    #[serde(default = "default_friendly_names")]
    pub friendly_names : std::collections::HashMap<String, String>,
    #[serde(default = "default_ignore_ipv4s")]
    pub ignore_ipv4s : Vec<String>,
    #[serde(default = "default_update_interval")]
    pub update_interval : u64,
    #[serde(default = "default_log_level")]
    pub log_level : String
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let mut input_file = std::fs::File::open("config.yml")?;
        let mut buf = Vec::new();
        input_file.read_to_end(&mut buf)?;
        Ok(serde_yaml::from_slice::<Config>(&buf)?)
    }

    pub fn get_notification_provider_config<T : DeserializeOwned>(provider_name : &str) -> Result<T, ConfigError> {
        let root_conf = Config::load().expect("Couldn't find config file");
        let config_raw = root_conf.notification_providers.get(provider_name).expect(format!("No {} config entry was found", provider_name).as_str());

        match serde_yaml::from_value::<T>(config_raw.clone()) {
            Ok(val) => Ok(val),
            Err(err) => Err(err.into())
        }
    }
}


fn default_providers() -> HashMap<String, serde_yaml::Value> {
    HashMap::new()
}

fn default_friendly_names() -> HashMap<String, String> {
    HashMap::new()
}

fn default_log_level() -> String {
    "info".to_owned()
}

fn default_ignore_ipv4s() -> Vec<String> {Vec::new()}


fn default_update_interval() -> u64 { 5 }

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("unknown config error")]
    Unknown,
    #[error("config error: {0:?}")]
    CustomError(Box<dyn std::error::Error>),
    #[error("config error: {0}")]
    Message(String),
    #[error("io error: {0:?}")]
    IoError(std::io::Error),
    #[error("yaml error: {0:?}")]
    YamlError(serde_yaml::Error)
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::YamlError(e)
    }
}