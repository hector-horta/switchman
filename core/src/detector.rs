//! Detección de navegadores: rutas conocidas (+ registro de Windows en `registry`).
use crate::config::Browser;
use std::path::Path;

/// (id, nombre, ruta con variables `%VAR%`).
const KNOWN: &[(&str, &str, &str)] = &[
    (
        "edge",
        "Microsoft Edge",
        r"%ProgramFiles(x86)%\Microsoft\Edge\Application\msedge.exe",
    ),
    (
        "chrome",
        "Google Chrome",
        r"%ProgramFiles%\Google\Chrome\Application\chrome.exe",
    ),
    (
        "chrome",
        "Google Chrome",
        r"%LOCALAPPDATA%\Google\Chrome\Application\chrome.exe",
    ),
    (
        "brave",
        "Brave",
        r"%ProgramFiles%\BraveSoftware\Brave-Browser\Application\brave.exe",
    ),
    (
        "firefox",
        "Mozilla Firefox",
        r"%ProgramFiles%\Mozilla Firefox\firefox.exe",
    ),
    ("zen", "Zen Browser", r"%ProgramFiles%\Zen Browser\zen.exe"),
    (
        "vivaldi",
        "Vivaldi",
        r"%LOCALAPPDATA%\Vivaldi\Application\vivaldi.exe",
    ),
    ("opera", "Opera", r"%LOCALAPPDATA%\Programs\Opera\opera.exe"),
    (
        "helium",
        "Helium",
        r"%LOCALAPPDATA%\imput\Helium\Application\chrome.exe",
    ),
];

/// Expande variables del formato como `%VAR%` en una cadena usando un proveedor de entorno.
pub fn expand_vars(template: &str, env: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('%')?;
        let name = &after[..end];
        if let Some(val) = env(name) {
            out.push_str(&val);
        } else {
            return None;
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// Devuelve una lista de navegadores conocidos cuya ruta expandida existe en el sistema,
/// ordenados según KNOWN y sin duplicar ids.
pub fn detect_known(
    env: &dyn Fn(&str) -> Option<String>,
    exists: &dyn Fn(&Path) -> bool,
) -> Vec<Browser> {
    let mut found: Vec<Browser> = Vec::new();
    for (id, name, template) in KNOWN {
        let Some(path) = expand_vars(template, env) else {
            continue;
        };
        if found.iter().any(|b| b.id == *id) || !exists(Path::new(&path)) {
            continue;
        };
        found.push(Browser {
            id: id.to_string(),
            name: name.to_string(),
            exe_path: path,
            args: Vec::new(),
        });
    }
    found
}

/// Combina múltiples listas de navegadores, manteniendo solo la primera aparición por id y ruta ejecutable (insensible a mayúsculas).
pub fn merge(lists: Vec<Vec<Browser>>) -> Vec<Browser> {
    let mut out: Vec<Browser> = Vec::new();
    for b in lists.into_iter().flatten() {
        let dup = out.iter().any(|o| {
            o.id.eq_ignore_ascii_case(&b.id) || o.exe_path.eq_ignore_ascii_case(&b.exe_path)
        });
        if !dup {
            out.push(b);
        }
    }
    out
}

#[cfg(test)]
#[path = "detector_tests.rs"]
mod tests;
