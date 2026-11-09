//! Errores del núcleo de ferro-link.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkError {
    #[error("URL inválida: {0}")]
    InvalidUrl(String),
    #[error("esquema no permitido: {0}")]
    UnsupportedScheme(String),
    #[error("patrón inválido: {0}")]
    InvalidPattern(String),
    #[error("configuración inválida: {0}")]
    Config(String),
    #[error("E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("navegador no encontrado: {0}")]
    BrowserNotFound(String),
    #[error("se rechazó lanzar ferro-link sobre sí mismo")]
    SelfLaunch,
}

pub type Result<T> = std::result::Result<T, LinkError>;
