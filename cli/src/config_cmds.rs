//! Subcomandos para la UI: `get-config`, `set-config --revision R` (JSON por stdin) y `status`.
//! Salida JSON por stdout; errores por stderr con código 1, 3 (config corrupta) o 4 (conflicto).
use serde_json::json;
use std::io::Read;
use switchman_core::config::Browser;
use switchman_core::config_types::Config;
use switchman_core::error::{LinkError, Result};
use switchman_core::{paths, registry, revision, store};
use tracing::info;

/// Tamaño máximo aceptado por stdin (`set-config`, `check-deeplink`).
const MAX_STDIN: u64 = 1024 * 1024;

/// Serializa `value` a JSON y lo imprime en una línea.
pub(crate) fn print_json(value: &impl serde::Serialize) -> Result<()> {
    let text = serde_json::to_string(value).map_err(|e| LinkError::Config(e.to_string()))?;
    println!("{text}");
    Ok(())
}

/// Lee de stdin (con tope) un JSON de tipo `T`.
pub(crate) fn read_stdin_json<T: serde::de::DeserializeOwned>() -> Result<T> {
    let mut text = String::new();
    std::io::stdin().take(MAX_STDIN).read_to_string(&mut text)?;
    serde_json::from_str(&text).map_err(|e| LinkError::Config(e.to_string()))
}

/// Imprime la versión y el contenido del archivo de configuración actual.
pub fn get_config() -> Result<()> {
    let versioned = revision::read_versioned(&paths::config_path()?)?;
    print_json(&versioned)
}

/// Guarda la configuración JSON recibida por stdin con la versión esperada.
pub fn set_config(expected_revision: &str) -> Result<()> {
    let config: Config = read_stdin_json()?;
    let revision = revision::save_versioned(&paths::config_path()?, &config, expected_revision)?;
    info!(%revision, "config saved from the UI");
    print_json(&json!({ "revision": revision }))
}

/// Devuelve el estado del registro, la ruta del ejecutable actual y el navegador de respaldo.
pub fn status() -> Result<()> {
    let fallback = store::read_json::<Browser>(&paths::fallback_path()?).ok();
    let self_exe = std::env::current_exe()?.display().to_string();
    print_json(
        &json!({ "registry": registry::status(), "self_exe": self_exe, "fallback": fallback }),
    )
}

/// Aparta un config.json corrupto como `.bak` e imprime la revisión para guardar de cero.
pub fn reset_config() -> Result<()> {
    let revision = revision::reset(&paths::config_path()?)?;
    info!("config.json moved aside as .bak");
    print_json(&json!({ "revision": revision }))
}
