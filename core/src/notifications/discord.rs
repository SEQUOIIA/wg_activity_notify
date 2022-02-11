use crate::notifications::{Event, NotificationHandler, NotificationData, Provider};
use serde::{Serialize, Deserialize};
use crate::{Config, ConfigError, ProviderError};
use crate::error::Error;

pub struct Discord {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiscordConfig {
    webhook_url : String,
    enable: bool
}

pub fn new() -> Result<Provider, ConfigError> {
    Ok(Provider {
        name: "Discord".to_string(),
        description: "".to_string(),
        config: Config::get_notification_provider_config("discord")?,
        handler: Some(Box::new(Discord {}))
    })
}

impl Discord {
    pub fn load_config() -> Result<DiscordConfig, ConfigError> {
        Config::get_notification_provider_config("discord")
    }
}

impl NotificationHandler for Discord {
    fn send(&self, data : NotificationData) -> Result<(), ProviderError> {
        let conf = Discord::load_config()?;
        let cli = reqwest::blocking::Client::new();
        let color = match data.event {
            Event::Connect => 6680723,
            Event::Disconnect => 14708848
        };

        let title = match data.event {
            Event::Connect => "New client connection",
            Event::Disconnect => "Client disconnected"
        };

        let payload = DiscordPayload {
            content: "".to_owned(),
            avatar_url: "".to_owned(),
            embeds: vec![
                Embed {
                    title: title.to_owned(),
                    description: data.msg,
                    url: "".to_owned(),
                    color,
                    author: Author { name: "wg-mgmt".to_owned() }
                }
            ]
        };

        cli.post(conf.webhook_url)
            .json(&payload)
            .send()
            .map_or_else(|e| { Err(ProviderError::ReqwestErr(e)) }, |_| { Ok(())})
    }

    fn get_provider(&self) -> Result<Provider, ConfigError> {
        new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordPayload {
    pub content: String,
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
    pub embeds: Vec<Embed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Embed {
    pub title: String,
    pub description: String,
    pub url: String,
    pub color: i64,
    pub author: Author,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
}
