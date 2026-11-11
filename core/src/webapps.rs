//! Webapps instaladas desde un navegador Chromium ("Instalar app"): son accesos directos del menú
//! Inicio que lanzan el navegador con `--app-id=<id>`. Se ofrecen como destinos más; para abrir la
//! URL dentro de la ventana de la app se agrega `--app-launch-url-for-shortcuts-menu-item={url}`.
use crate::config::Browser;
use crate::launcher::URL_PLACEHOLDER;
use crate::registry::slug;
use std::path::{Path, PathBuf};

/// Prefijo del argumento que identifica una webapp de Chromium.
const APP_ID: &str = "--app-id=";
/// Profundidad máxima al recorrer el menú Inicio (Programs\<Navegador> Apps\app.lnk).
const MAX_DEPTH: usize = 3;

/// Divide una cadena de comandos en argumentos para Windows.
pub fn split_args(cmd: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in cmd.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Convierte una ruta de acceso directo a una webapp Chromium en un objeto Browser válido.
pub fn from_shortcut(
    name: &str,
    folder: Option<&str>,
    target: &str,
    args: &str,
) -> Option<Browser> {
    let target = target.trim();
    if target.is_empty() {
        return None;
    }
    let mut parts = split_args(args);
    let has_app_id = parts
        .iter()
        .any(|a| a.strip_prefix(APP_ID).is_some_and(|id| !id.is_empty()));
    if !has_app_id {
        return None;
    }

    let label = folder.and_then(|f| f.strip_suffix(" Apps"));
    let display = match label {
        Some(l) => format!("{name} (app de {l})"),
        None => format!("{name} (app)"),
    };

    let mut base = slug(name);
    base.truncate(60);
    let id = format!("{}-app", base.trim_end_matches('-'));

    parts.push(format!(
        "--app-launch-url-for-shortcuts-menu-item={URL_PLACEHOLDER}"
    ));
    Some(Browser {
        id,
        name: display,
        exe_path: target.to_string(),
        args: parts,
    })
}

/// Recorre recursivamente una carpeta para encontrar archivos `.lnk` de acceso directo a webapps Chromium.
fn shortcuts(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            shortcuts(&path, depth + 1, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
        {
            out.push(path);
        }
    }
}

/// Lee un acceso directo `.lnk` para convertirlo en un objeto `Browser` válido si es una webapp Chromium.
fn read_shortcut(path: &Path) -> Option<Browser> {
    let link = lnk::ShellLink::open(path, lnk::encoding::WINDOWS_1252).ok()?;
    let args = link
        .string_data()
        .command_line_arguments()
        .clone()
        .unwrap_or_default();
    if !args.contains(APP_ID) {
        return None;
    }
    let target = link
        .link_info()
        .as_ref()
        .and_then(|i| i.local_base_path())
        .map(str::to_string)?;
    if !Path::new(&target).is_file() {
        return None;
    }
    let name = path.file_stem()?.to_string_lossy().to_string();
    let folder = path
        .parent()
        .and_then(Path::file_name)
        .map(|f| f.to_string_lossy().to_string());
    from_shortcut(&name, folder.as_deref(), &target, &args)
}

/// Obtiene todas las webapps de Chromium instaladas como accesos directos en el menú Inicio del sistema.
pub fn installed_webapps() -> Vec<Browser> {
    let mut roots = Vec::new();
    for var in ["APPDATA", "PROGRAMDATA"] {
        if let Ok(base) = std::env::var(var) {
            roots.push(PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
    }

    let mut files = Vec::new();
    for root in &roots {
        shortcuts(root, 0, &mut files);
    }

    let mut found: Vec<Browser> = Vec::new();
    for file in &files {
        if let Some(app) = read_shortcut(file) {
            if !found.iter().any(|b| b.id == app.id) {
                found.push(app);
            }
        }
    }

    found
}

#[cfg(test)]
#[path = "webapps_tests.rs"]
mod tests;
