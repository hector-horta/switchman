//! Modelo de configuración (`config.json`) y navegador de respaldo (`fallback.json`).
use crate::config_types::Config;
use crate::error::{LinkError, Result};
use crate::matcher::HostPattern;
use crate::store;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Browser {
    pub id: String,
    pub name: String,
    pub exe_path: String,
    #[serde(default)]
    pub args: Vec<String>,
}

/// Resultado de cargar la configuración efectiva.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loaded {
    /// `config.json` válido.
    Config(Config),
    /// `config.json` falta o es inválido: se usa el navegador de respaldo.
    Fallback(Browser),
}

const MAX_ID_LEN: usize = 64;

/// Verifica que el ID cumple los requisitos: no vacío, longitud ≤ 64 caracteres,
/// y solo contiene letras numéricas ASCII o guiones bajos, puntos o guiones.
pub(crate) fn check_id(kind: &str, id: &str) -> Result<()> {
    let ok = !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if ok {
        Ok(())
    } else {
        Err(LinkError::Config(format!("{kind} inválido: {id:?}")))
    }
}

/// Valida toda la configuración (`Config`) asegurando que los IDs cumplen los requisitos,
/// las rutas de navegadores no están vacías y los patrones de host son válidos.
pub fn validate(config: &Config) -> Result<()> {
    check_id("navegador por defecto", &config.default_browser_id)?;
    for rule in &config.rules {
        check_id("regla", &rule.id)?;
        check_id("navegador de la regla", &rule.browser_id)?;
        HostPattern::parse(&rule.pattern)?;
    }
    for b in &config.custom_browsers {
        check_id("navegador", &b.id)?;
        if b.exe_path.trim().is_empty() {
            return Err(LinkError::Config(format!("ruta vacía para {}", b.id)));
        }
    }
    crate::deeplinks::validate(config)
}

/// Carga y valida la configuración desde un archivo JSON en el path proporcionado.
pub fn load(path: &Path) -> Result<Config> {
    let config: Config = store::read_json(path)?;
    validate(&config)?;
    Ok(config)
}

/// Guarda la configuración en un archivo JSON de forma atómica tras validarla.
pub fn save(path: &Path, config: &Config) -> Result<()> {
    validate(config)?;
    store::write_json_atomic(path, config)
}

/// Guarda el navegador de respaldo en un archivo JSON atómico tras validar su ID y ruta.
pub fn save_fallback(path: &Path, browser: &Browser) -> Result<()> {
    check_id("navegador de respaldo", &browser.id)?;
    if browser.exe_path.trim().is_empty() {
        return Err(LinkError::Config(
            "ruta vacía en el navegador de respaldo".into(),
        ));
    }
    store::write_json_atomic(path, browser)
}

/// Carga la configuración efectiva: primero intenta leer y validar el archivo JSON principal.
/// Si falla (falta o es inválido), carga el navegador de respaldo válidamente identificado.
pub fn load_effective(config_path: &Path, fallback_path: &Path) -> Result<Loaded> {
    match load(config_path) {
        Ok(c) => return Ok(Loaded::Config(c)),
        Err(e) => {
            warn!(error = %e, "config.json inválido o ausente; usando el navegador de respaldo")
        }
    }
    let browser: Browser = store::read_json(fallback_path)?;
    check_id("navegador de respaldo", &browser.id)?;
    Ok(Loaded::Fallback(browser))
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
