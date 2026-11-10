//! `config.json` con revisión: control optimista de concurrencia para la UI.
//! La UI lee `{revision, config}` y al guardar devuelve la revisión leída; si el archivo
//! cambió entretanto (edición a mano, otra instancia), se rechaza con `LinkError::Conflict`.
use crate::config;
use crate::config_types::Config;
use crate::error::{LinkError, Result};
use serde::Serialize;
use std::hash::{DefaultHasher, Hasher};
use std::path::Path;

/// Revisión de un `config.json` que todavía no existe.
pub const NO_FILE: &str = "none";

/// Configuración junto con la revisión del archivo del que se leyó.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Versioned {
    pub revision: String,
    pub config: Config,
}

/// Revisión opaca del contenido; solo se compara dentro del mismo ejecutable.
pub fn revision_of(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    format!("{:016x}", hasher.finish())
}

/// Lee los bytes del archivo en la ruta proporcionada. Devuelve `None` si el archivo no existe.
fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(LinkError::Io(e)),
    }
}

/// lee config.json con su revisión; si falta, config vacía con NO_FILE; JSON inválido o config inválida -> Corrupt.
pub fn read_versioned(path: &Path) -> Result<Versioned> {
    let Some(bytes) = read_bytes(path)? else {
        return Ok(Versioned {
            revision: NO_FILE.to_string(),
            config: Config::default(),
        });
    };
    let config: Config =
        serde_json::from_slice(&bytes).map_err(|e| LinkError::Corrupt(e.to_string()))?;
    config::validate(&config).map_err(|e| LinkError::Corrupt(e.to_string()))?;
    Ok(Versioned {
        revision: revision_of(&bytes),
        config,
    })
}

/// Guarda el config solo si la revisión en disco coincide con la esperada; devuelve la nueva revisión.
pub fn save_versioned(path: &Path, config: &Config, expected: &str) -> Result<String> {
    config::validate(config)?;
    let current = read_bytes(path)?.map_or_else(|| NO_FILE.to_string(), |b| revision_of(&b));
    if current != expected {
        return Err(LinkError::Conflict);
    }
    config::save(path, config)?;
    let written = read_bytes(path)?
        .ok_or_else(|| LinkError::Config("config.json desapareció tras guardar".into()))?;
    Ok(revision_of(&written))
}

/// Aparta `config.json` (p. ej. corrupto) como `config.json.bak` para empezar de cero.
/// Devuelve la revisión con la que guardar a continuación (`NO_FILE`).
pub fn reset(path: &Path) -> Result<String> {
    let mut bak = path.as_os_str().to_owned();
    bak.push(".bak");
    match std::fs::rename(path, std::path::PathBuf::from(bak)) {
        Ok(()) => Ok(NO_FILE.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(NO_FILE.to_string()),
        Err(e) => Err(LinkError::Io(e)),
    }
}

#[cfg(test)]
#[path = "revision_tests.rs"]
mod tests;
