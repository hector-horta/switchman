//! Errores del núcleo de switchman.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("scheme not allowed: {0}")]
    UnsupportedScheme(String),
    #[error("invalid pattern: {0}")]
    InvalidPattern(String),
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("browser not found: {0}")]
    BrowserNotFound(String),
    #[error("refused to launch switchman on itself")]
    SelfLaunch,
    #[error("config.json is damaged: {0}")]
    Corrupt(String),
    #[error("config.json changed since it was read; reload before saving")]
    Conflict,
}

pub type Result<T> = std::result::Result<T, LinkError>;
