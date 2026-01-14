use std::collections::HashMap;
use std::fmt::format;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_yaml::Value;
use tracing::{debug, info};
use crate::config::{Config, ConfigError};
use crate::notifications::{Event, init_providers, init_providers_map, NotificationData, ProviderError};
use crate::wg::{get_dump, WgEntry, WgError};
use error::Error;
use std::net::SocketAddr;

pub mod config;
pub mod wg;
pub mod notifications;
pub mod error;

pub struct Daemon {
    entries : HashMap<String, WgEntry>,
    last_handshake: HashMap<String, u64>,
    last_known_endpoint: HashMap<String, String>,
    status: HashMap<String, Status>,
    conf : Config
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    pub is_disconnected : bool
}

impl Daemon {
    pub fn new(conf : Config) -> Self {
        Self {
            entries: HashMap::new(),
            last_handshake: HashMap::new(),
            last_known_endpoint: HashMap::new(),
            status: HashMap::new(),
            conf,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_int();
            std::thread::sleep(std::time::Duration::from_secs(self.conf.update_interval));
        }
    }

    fn send_notification(&self, data : NotificationData) -> error::Result<()> {
        let providers_conf = self.conf.notification_providers.clone();
        std::thread::spawn(move || {
            let providers = init_providers_map().unwrap();

            for (key, provider) in providers {
                if providers_conf.contains_key(&key) {
                    if provider.enabled() {
                        debug!("Sending notification via {} provider", key);
                        provider.send(data.clone()).unwrap();
                    }
                }
            }
        });

        Ok(())
    }

    fn get_friendly_name(&self, pub_key : &str) -> String {
        return match self.conf.friendly_names.get(pub_key) {
            None => pub_key.to_owned(),
            Some(val) => format!("{val} ({pub_key})")
        }
    }

    fn status_of_entry(&self, entry : &WgEntry) -> error::Result<Status> {
        let mut status = Status::default();

        if let WgEntry::Client(data) = entry {
            let current_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time is an illusion").as_secs();
            let handshake_threshold = data.persistent_keepalive * 7;
            let seconds_since_last_handshake = current_epoch - data.latest_handshake;


            if seconds_since_last_handshake > handshake_threshold {
                status.is_disconnected = true;
            }

            return Ok(status);
        };

        Err(Error::UnableToGetStatusOfEntry())
    }

    fn should_ignore(&self, endpoint: &Option<String>) -> bool {
        if let Some(ep) = endpoint {
            if let Ok(addr) = ep.parse::<SocketAddr>() {
                let ip = addr.ip();
                return self.conf.ignored_subnets.iter().any(|subnet| subnet.contains(&ip));
            }
        }
        false
    }

    fn run_int(&mut self) {
        debug!("Checking WireGuard clients");

        let entries = get_dump();
        for entry in &entries {
            if let WgEntry::Client(data) = entry {
                self.entries.insert(data.public_key.clone(), entry.clone());

                let mut data_ip = "?".to_owned();

                if let Some(endpoint) = &data.endpoint {
                    self.last_known_endpoint.insert(data.public_key.clone(), endpoint.clone());
                    data_ip = endpoint.clone();
                } else {
                    if let Some (endpoint) = self.last_known_endpoint.get(&data.public_key) {
                        data_ip = endpoint.clone();
                    }
                }

                let current_status = self.status_of_entry(entry).unwrap();
                let previous_status = self.status.get(&data.public_key);
                let friendly_name = self.get_friendly_name(&data.public_key);

                if current_status.is_disconnected {
                    if let Some(s) = previous_status {
                        if current_status.is_disconnected != s.is_disconnected { // Reached if current is_disconnected is true & the previous status is not
                            let msg = format!("Client {} using endpoint {} has disconnected", friendly_name, data_ip);
                            info!("{}", msg);
                            if !self.should_ignore(&data.endpoint) {
                                self.send_notification(NotificationData { msg, event: Event::Disconnect });
                            }
                        }
                    }
                } else {
                    if let Some(s) = previous_status {
                        if current_status.is_disconnected != s.is_disconnected { // Reached if current is_disconnected is false & the previous status is not
                            let msg = format!("Client {} using endpoint {} has connected", friendly_name, data_ip);
                            info!("{}", msg);
                            if !self.should_ignore(&data.endpoint) {
                                self.send_notification(NotificationData { msg, event: Event::Connect });
                            }
                        }
                    }
                }

                // Update last_handshake & status
                self.last_handshake.insert(data.public_key.clone(), data.latest_handshake);
                self.status.insert(data.public_key.clone(), current_status.clone());
            }
        }

    }
}