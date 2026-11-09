//! Rutas de datos de ferro-link bajo `%LOCALAPPDATA%\FerroLink`.
use crate::error::{LinkError, Result};
use std::path::PathBuf;

const APP_DIR: &str = "FerroLink";

/// Devuelve la ruta absoluta al directorio de datos de FerroLink en `%LOCALAPPDATA%\FerroLink`.
pub fn data_dir() -> Result<PathBuf> {
    let raw = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| LinkError::Config("LOCALAPPDATA no definida".into()))?;
    let base = PathBuf::from(raw);
    if !base.is_absolute() {
        Err(LinkError::Config(format!(
            "LOCALAPPDATA no es absoluta: {}",
            base.display()
        )))
    } else {
        Ok(base.join(APP_DIR))
    }
}

/// Configuración principal editada por la UI.
pub fn config_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("config.json"))
}

/// Navegador de respaldo usado cuando `config.json` falta o está corrupto.
pub fn fallback_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("fallback.json"))
}

/// Registro de eventos del CLI.
pub fn log_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("ferro-link.log"))
}
