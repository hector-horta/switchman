//! Lanza el navegador elegido directamente (sin shell: nunca `cmd /c start`).
use crate::config::Browser;
use crate::error::{LinkError, Result};
use std::path::Path;
use std::process::Command;
use url::Url;

/// Normaliza una ruta de archivo para lanzar un navegador en Windows,
/// reemplazando los caracteres especiales y ajustando el prefijo verbatim.
fn normalize(p: &Path) -> String {
    let canon = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let text = canon
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    text.strip_prefix(VERBATIM_PREFIX)
        .map(str::to_string)
        .unwrap_or(text)
}

/// Determina si dos rutas de archivos apuntan al mismo archivo en el sistema,
/// comparando sus representaciones normalizadas para compatibilidad con navegadores.
pub fn same_exe(a: &Path, b: &Path) -> bool {
    normalize(a) == normalize(b)
}

/// Prefijo de rutas verbatim que agrega `canonicalize` en Windows.
const VERBATIM_PREFIX: &str = r"\\?\";

/// Marcador que se reemplaza por la URL dentro de un argumento (p. ej. webapps de Chromium:
/// `--app-launch-url-for-shortcuts-menu-item={url}`).
pub const URL_PLACEHOLDER: &str = "{url}";

/// Convierte los argumentos del navegador en una lista de strings con la URL reemplazada si aparece el marcador `{url}`.
pub fn launch_args(args: &[String], url: &str) -> Vec<String> {
    if args.iter().any(|a| a.contains(URL_PLACEHOLDER)) {
        args.iter()
            .map(|a| a.replace(URL_PLACEHOLDER, url))
            .collect()
    } else {
        let mut out = args.to_vec();
        out.push(url.to_string());
        out
    }
}

/// Lanza el navegador especificado con la URL proporcionada, verificando primero que no se lance a sí mismo.
pub fn launch(browser: &Browser, url: &Url, self_exe: &Path) -> Result<()> {
    let exe = Path::new(browser.exe_path.trim());
    if same_exe(exe, self_exe) {
        return Err(LinkError::SelfLaunch);
    }
    if !exe.is_file() {
        return Err(LinkError::BrowserNotFound(browser.exe_path.clone()));
    }
    Command::new(exe)
        .args(launch_args(&browser.args, url.as_str()))
        .spawn()?;
    Ok(())
}

#[cfg(test)]
#[path = "launcher_tests.rs"]
mod tests;
