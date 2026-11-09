//! Lectura y escritura atómica de archivos JSON (escribe a `.tmp` y renombra).
use crate::error::{LinkError, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::Path;

/// Lee un archivo JSON en formato seguro y lo deserializa en el tipo T.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str::<T>(&text)
        .map_err(|e| LinkError::Config(format!("{}: {}", path.display(), e)))
}

/// Escribe un valor JSON de forma atómica en un archivo: primero se crea un archivo temporal con el contenido,
/// luego se renombra y finalmente se devuelve el resultado si todo sale bien.
pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| LinkError::Config(e.to_string()))?;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    fs::write(&tmp, text)?;
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(LinkError::Io(e));
    }
    Ok(())
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
