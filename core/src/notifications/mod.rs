use std::collections::HashMap;
use reqwest::blocking::Response;
use reqwest::Error;
use serde::{Serialize, Deserialize};
use crate::ConfigError;
use thiserror::Error;

pub mod discord;
pub mod pushover;

#[derive(Clone, Debug)]
pub struct NotificationData {
    pub msg : String,
    pub event : Event
}

#[derive(Clone, Debug)]
pub enum Event {
    Connect,
    Disconnect
}

pub trait NotificationHandler {
    fn send(&self, data : NotificationData) -> Result<(), ProviderError>;
    fn get_provider(&self) -> Result<Provider, ConfigError>;
}

pub fn init_providers() -> Result<Vec<Provider>, ConfigError> {
    let mut providers = Vec::new();

    providers.push(discord::new()?);
    providers.push(pushover::new()?);

    Ok(providers)
}

pub fn init_providers_map() -> Result<HashMap<String, Provider>, ConfigError> {
    let mut payload = HashMap::new();
    for provider in init_providers()? {
        payload.insert(provider.name.to_lowercase(), provider);
    };

    Ok(payload)
}

pub fn does_provider_exist(val : &str) -> bool {
    init_providers_map().unwrap().contains_key(val)
}

#[derive(Serialize, Deserialize)]
pub struct Provider {
    pub name : String,
    #[serde(skip)]
    pub description : String,
    #[serde(skip)]
    pub config : std::collections::HashMap<String, serde_yaml::Value>,
    #[serde(skip)]
    pub handler : Option<Box<dyn NotificationHandler>>
}

impl Clone for Provider {
    fn clone(&self) -> Self {
        self.handler.as_ref().expect("No handler was provided for Provider").get_provider().unwrap()
    }
}

impl Provider {
    pub fn send(&self, data : NotificationData) -> Result<(), ProviderError> {
        return match self.handler.as_ref() {
            Some(handler) => {
                handler.send(data)
            },
            None => Ok(())
        }
    }

    pub fn enabled(&self) -> bool {
        return match self.config.get("enable") {
            Some(val) => {
                return match val.as_bool() {
                    Some(enable) => enable,
                    None => false
                }
            },
            None => false
        }
    }
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("config error: {0:?}")]
    CustomError(Box<dyn std::error::Error>),
    #[error("error: {0}")]
    Message(String),
    #[error("config error: {0:?}")]
    ConfigErr(ConfigError),
    #[error("reqwest error: {0:?}")]
    ReqwestErr(reqwest::Error),
}

impl From<ConfigError> for ProviderError {
    fn from(e: ConfigError) -> Self {
        Self::ConfigErr(e)
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestErr(e)
    }
}