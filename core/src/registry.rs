//! Registro de ferro-link como navegador (solo HKCU) y lectura de navegadores instalados.
use crate::config::Browser;
use crate::error::Result;
use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

pub const APP_NAME: &str = "FerroLink";
const PROG_ID: &str = "FerroLinkURL";
const CLIENTS: &str = r"Software\Clients\StartMenuInternet";

/// Extrae la ruta completa del `.exe` de una línea de comandos del registro (con o sin comillas).
pub fn exe_from_command(cmd: &str) -> Option<String> {
    let cmd = cmd.trim();
    if let Some(rest) = cmd.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string()).filter(|s| !s.is_empty());
    }
    let lower = cmd.to_ascii_lowercase();
    let end = lower.find(".exe")? + 4;
    Some(cmd[..end].to_string())
}

/// Convierte un nombre de clave del registro en una ID válida: letras/minúsculas ASCII y dígitos,
/// separadores `-` únicos, máximo 64 caracteres.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.truncate(64);
    out.trim_end_matches('-').to_string()
}

/// Registra FerroLink como navegador en HKEY_CURRENT_USER, configurando la clave de asociación de URL y el menú de inicio.
pub fn register(exe: &Path) -> Result<()> {
    let exe = exe.display().to_string();
    let cmd = format!("\"{exe}\" open-url \"%1\"");
    let icon = format!("{exe},0");
    let client = format!(r"{CLIENTS}\{APP_NAME}");
    let caps = format!(r"{client}\Capabilities");
    let classes = format!(r"Software\Classes\{PROG_ID}");
    let entries: Vec<(String, &str, String)> = vec![
        (classes.clone(), "", "Ferro Link URL".to_string()),
        (format!(r"{classes}\DefaultIcon"), "", icon.clone()),
        (format!(r"{classes}\shell\open\command"), "", cmd.clone()),
        (client.clone(), "", "Ferro Link".to_string()),
        (format!(r"{client}\DefaultIcon"), "", icon.clone()),
        (
            format!(r"{client}\shell\open\command"),
            "",
            format!("\"{exe}\""),
        ),
        (caps.clone(), "ApplicationName", "Ferro Link".to_string()),
        (
            caps.clone(),
            "ApplicationDescription",
            "Enrutador de enlaces que quita parámetros de rastreo".to_string(),
        ),
        (caps.clone(), "ApplicationIcon", icon),
        (
            format!(r"{caps}\URLAssociations"),
            "http",
            PROG_ID.to_string(),
        ),
        (
            format!(r"{caps}\URLAssociations"),
            "https",
            PROG_ID.to_string(),
        ),
        (
            r"Software\RegisteredApplications".to_string(),
            APP_NAME,
            caps.clone(),
        ),
    ];
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for (path, name, value) in &entries {
        let (key, _) = hkcu.create_subkey(path)?;
        key.set_value(name, value)?;
    }
    Ok(())
}

/// Elimina todo lo registrado por FerroLink en HKEY_CURRENT_USER.
pub fn unregister() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for path in [
        format!(r"Software\Classes\{PROG_ID}"),
        format!(r"{CLIENTS}\{APP_NAME}"),
    ] {
        match hkcu.delete_subkey_all(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }

    if let Ok(apps) = hkcu.open_subkey_with_flags(
        r"Software\RegisteredApplications",
        winreg::enums::KEY_SET_VALUE,
    ) {
        match apps.delete_value(APP_NAME) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

/// Devuelve una lista de navegadores instalados en Windows registrados bajo StartMenuInternet,
/// excluyendo FerroLink y omitiendo entradas con rutas de ejecutable inválidas.
pub fn installed_browsers() -> Vec<Browser> {
    let mut found = Vec::new();
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let Ok(clients) = RegKey::predef(root).open_subkey(CLIENTS) else {
            continue;
        };
        for key_name in clients.enum_keys().flatten() {
            if key_name.eq_ignore_ascii_case(APP_NAME) {
                continue;
            };
            let Ok(entry) = clients.open_subkey(&key_name) else {
                continue;
            };
            let Ok(command) = entry
                .open_subkey(r"shell\open\command")
                .and_then(|k| k.get_value::<String, _>(""))
            else {
                continue;
            };
            let Some(exe_path) = exe_from_command(&command) else {
                continue;
            };
            if !Path::new(&exe_path).is_file() {
                continue;
            };
            let name = entry
                .get_value::<String, _>("")
                .unwrap_or_else(|_| key_name.clone());
            found.push(Browser {
                id: slug(&key_name),
                name,
                exe_path,
                args: Vec::new(),
            });
        }
    }
    found
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
