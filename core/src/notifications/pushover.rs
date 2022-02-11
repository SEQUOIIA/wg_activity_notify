use crate::notifications::{Event, NotificationHandler, NotificationData, Provider};
use serde::{Serialize, Deserialize};
use crate::{Config, ConfigError, ProviderError};
use crate::error::Error;

pub struct Pushover {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PushoverConfig {
    api_key : String,
    device_key : String,
    #[serde(default = "default_priority")]
    priority: i32,
    enable: bool
}

fn default_priority() -> i32 {
    1
}

pub fn new() -> Result<Provider, ConfigError> {
    Ok(Provider {
        name: "Pushover".to_string(),
        description: "".to_string(),
        config: Config::get_notification_provider_config("pushover")?,
        handler: Some(Box::new(Pushover {}))
    })
}

impl Pushover {
    pub fn load_config() -> Result<PushoverConfig, ConfigError> {
        Config::get_notification_provider_config("pushover")
    }
}

impl NotificationHandler for Pushover {
    fn send(&self, data : NotificationData) -> Result<(), ProviderError> {
        let conf = Pushover::load_config()?;
        let cli = reqwest::blocking::Client::new();

        let title = match data.event {
            Event::Connect => "New client connection",
            Event::Disconnect => "Client disconnected"
        };

        let payload = PushoverPayload {
            token: conf.api_key.clone(),
            user: conf.device_key.clone(),
            title: title.to_owned(),
            message: data.msg.clone(),
            priority: conf.priority
        };

        cli.post("https://api.pushover.net/1/messages.json")
            .json(&payload)
            .send()
            .map_or_else(|e| { Err(ProviderError::ReqwestErr(e)) }, |_| { Ok(())})
    }

    fn get_provider(&self) -> Result<Provider, ConfigError> {
        new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PushoverPayload {
    pub token: String,
    pub user: String,
    pub title: String,
    pub message: String,
    pub priority: i32,
}