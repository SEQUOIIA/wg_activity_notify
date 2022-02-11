use thiserror::Error;
use crate::{ConfigError, ProviderError, WgError};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to get status of entry")]
    UnableToGetStatusOfEntry(),
    #[error("config error: {0:?}")]
    CustomError(Box<dyn std::error::Error>),
    #[error("error: {0}")]
    Message(String),
    #[error("config error: {0:?}")]
    ConfigErr(ConfigError),
    #[error("wg error: {0:?}")]
    WgErr(WgError),
    #[error("wg error: {0:?}")]
    ProviderErr(ProviderError)
}

impl From<ConfigError> for Error {
    fn from(e: ConfigError) -> Self {
        Self::ConfigErr(e)
    }
}

impl From<WgError> for Error {
    fn from(e: WgError) -> Self {
        Self::WgErr(e)
    }
}

impl From<ProviderError> for Error {
    fn from(e: ProviderError) -> Self {
        Self::ProviderErr(e)
    }
}
