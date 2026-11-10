//! Registro de switchman como navegador (solo HKCU) y lectura de navegadores instalados.
use crate::config::Browser;
use crate::error::Result;
use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

pub const APP_NAME: &str = "Switchman";
const PROG_ID: &str = "SwitchmanURL";
const CLIENTS: &str = r"Software\Clients\StartMenuInternet";
const URL_ASSOC: &str = r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations";

/// Estado del registro de Switchman en Windows (lo muestra la UI).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RegStatus {
    /// Ejecutable registrado en `SwitchmanURL`, si existe.
    pub registered_exe: Option<String>,
    pub default_http: bool,
    pub default_https: bool,
}

/// Extrae la ruta completa del `.exe` de una línea de comandos del registro (con o sin comillas).
pub fn exe_from_command(cmd: &str) -> Option<String> {
    let cmd = cmd.trim();
    if let Some(rest) = cmd.strip_prefix('"') {
        let (exe, _) = rest.split_once('"')?;
        return Some(exe.to_string()).filter(|s| !s.is_empty());
    }
    let end = cmd.to_ascii_lowercase().find(".exe")? + 4;
    cmd.get(..end).map(str::to_string)
}

/// Elimina el sufijo de instalación (hash o número) si cumple los criterios: contiene al menos un dígito y tiene longitud ≥12.
fn strip_install_suffix(name: &str) -> &str {
    name.rsplit_once(['-', '.']).map_or(name, |(head, tail)| {
        if head.is_empty()
            || tail.len() < 12
            || !tail.chars().all(|c| c.is_ascii_alphanumeric())
            || !tail.chars().any(|c| c.is_ascii_digit())
        {
            name
        } else {
            head
        }
    })
}

/// Convierte un nombre de clave del registro en una ID válida: letras/minúsculas ASCII y dígitos,
/// separadores `-` únicos, máximo 64 caracteres.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in strip_install_suffix(name.trim()).chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.truncate(64);
    out.trim_end_matches('-').to_string()
}

/// Registra Switchman como navegador en HKEY_CURRENT_USER, configurando la clave de asociación de URL y el menú de inicio.
pub fn register(exe: &Path) -> Result<()> {
    let exe = exe.display().to_string();
    let cmd = format!("\"{exe}\" open-url \"%1\"");
    let icon = format!("{exe},0");
    let client = format!(r"{CLIENTS}\{APP_NAME}");
    let caps = format!(r"{client}\Capabilities");
    let classes = format!(r"Software\Classes\{PROG_ID}");
    let entries: Vec<(String, &str, String)> = vec![
        (classes.clone(), "", "Switchman URL".to_string()),
        (format!(r"{classes}\DefaultIcon"), "", icon.clone()),
        (format!(r"{classes}\shell\open\command"), "", cmd.clone()),
        (client.clone(), "", "Switchman".to_string()),
        (format!(r"{client}\DefaultIcon"), "", icon.clone()),
        (
            format!(r"{client}\shell\open\command"),
            "",
            format!("\"{exe}\""),
        ),
        (caps.clone(), "ApplicationName", "Switchman".to_string()),
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

/// Elimina todo lo registrado por Switchman en HKEY_CURRENT_USER.
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
/// excluyendo Switchman y omitiendo entradas con rutas de ejecutable inválidas.
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

/// Obtiene el ID de programa asociado a un esquema de URL en Windows, priorizando la clave para el último usuario.
fn default_prog_id(hkcu: &RegKey, scheme: &str) -> Option<String> {
    let base = format!(r"{URL_ASSOC}\{scheme}");
    hkcu.open_subkey(format!(r"{base}\UserChoiceLatest\ProgId"))
        .and_then(|k| k.get_value::<String, _>("ProgId"))
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            hkcu.open_subkey(format!(r"{base}\UserChoice"))
                .and_then(|k| k.get_value::<String, _>("ProgId"))
                .ok()
                .filter(|s| !s.is_empty())
        })
}

/// Obtiene el estado actual del registro de Switchman en HKEY_CURRENT_USER.
pub fn status() -> RegStatus {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let registered_exe = hkcu
        .open_subkey(format!(r"Software\Classes\{PROG_ID}\shell\open\command"))
        .and_then(|k| k.get_value::<String, _>(""))
        .ok()
        .and_then(|cmd| exe_from_command(&cmd));

    let default_http =
        default_prog_id(&hkcu, "http").is_some_and(|p| p.eq_ignore_ascii_case(PROG_ID));
    let default_https =
        default_prog_id(&hkcu, "https").is_some_and(|p| p.eq_ignore_ascii_case(PROG_ID));

    RegStatus {
        registered_exe,
        default_http,
        default_https,
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
