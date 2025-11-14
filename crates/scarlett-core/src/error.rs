//! Error types for Scarlett operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("USB error: {0}")]
    Usb(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Not supported by this device: {0}")]
    NotSupported(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
