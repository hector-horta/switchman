//! Protocolos de apps registrados en Windows (`whatsapp:`, `discord:`…) y apertura de URIs.
//! Una clave de HKEY_CLASSES_ROOT con el valor "URL Protocol" es un protocolo que abre una app.
use crate::error::{LinkError, Result};
use winreg::enums::HKEY_CLASSES_ROOT;
use winreg::RegKey;

/// Valor que marca una clave de clase como protocolo.
const URL_PROTOCOL: &str = "URL Protocol";
/// Protocolos del sistema o de navegadores que no tiene sentido ofrecer como apps.
const SYSTEM_SCHEMES: &[&str] = &[
    "http",
    "https",
    "ftp",
    "file",
    "mailto",
    "tel",
    "callto",
    "sip",
    "sips",
    "im",
    "irc",
    "news",
    "nntp",
    "snews",
    "telnet",
    "rlogin",
    "tn3270",
    "ldap",
    "res",
    "about",
    "javascript",
    "vbscript",
    "data",
    "mk",
    "its",
    "search",
    "shell",
    "read",
    "feed",
    "feeds",
    "webcal",
    "xmpp",
    "microsoft-edge",
    "tvaudio",
    "slack-teams",
];

/// Determina si el esquema dado es un protocolo del sistema o de navegadores.
pub fn is_system_scheme(scheme: &str) -> bool {
    let s = scheme.to_ascii_lowercase();
    SYSTEM_SCHEMES.contains(&s.as_str())
        || s.starts_with("ms-")
        || s.starts_with("microsoft")
        || s.starts_with("windows")
}

/// Devuelve true si el esquema está registrado como protocolo de apertura en Windows.
pub fn scheme_registered(scheme: &str) -> bool {
    if scheme.is_empty()
        || !scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
    {
        return false;
    }
    RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey(scheme)
        .and_then(|key| key.get_raw_value(URL_PROTOCOL))
        .is_ok()
}

/// Devuelve una lista de protocolos registrados como aplicaciones en Windows que no son esquemas del sistema,
/// ordenados alfabéticamente y sin duplicados.
pub fn registered_schemes() -> Vec<String> {
    let root = RegKey::predef(HKEY_CLASSES_ROOT);
    let mut out: Vec<String> = root
        .enum_keys()
        .flatten()
        .filter(|name| !name.starts_with('.') && !is_system_scheme(name))
        .filter(|name| {
            root.open_subkey(name)
                .and_then(|key| key.get_raw_value(URL_PROTOCOL))
                .is_ok()
        })
        .map(|name| name.to_ascii_lowercase())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Abre una URI con el explorador de Windows usando el protocolo registrado.
pub fn open_uri(uri: &str) -> Result<()> {
    let Some((scheme, _)) = uri.split_once(':') else {
        return Err(LinkError::InvalidUrl(format!("sin protocolo: {}", uri)));
    };
    if is_system_scheme(scheme) || scheme.is_empty() {
        return Err(LinkError::UnsupportedScheme(scheme.to_string()));
    }

    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wide = |s: &str| {
        s.encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>()
    };
    let (verb, target) = (wide("open"), wide(uri));

    // SAFETY: cadenas UTF-16 terminadas en 0 que viven durante la llamada; hwnd nulo es válido.
    let code = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;
    if code > 32 {
        Ok(())
    } else {
        Err(LinkError::Io(std::io::Error::other(format!(
            "ShellExecuteW devolvió {} para {}",
            code, scheme
        ))))
    }
}

/// Familias de protocolos de Windows, navegadores o herramientas que no se listan como apps.
const NOISE_PREFIXES: &[&str] = &[
    "xbox",
    "winscp",
    "onenote",
    "ms",
    "ie",
    "explorer",
    "wmp",
    "vs",
    "bing",
    "hp-",
    "gaming",
    "search",
    "firefoxurl",
    "google-chrome",
    "brave-browser",
    "opera",
    "vivaldi",
    "chrome",
    "edge",
    "web+",
];
/// Protocolos de red, archivos o del sistema que tampoco son apps a las que mandar enlaces.
const NOISE_SCHEMES: &[&str] = &[
    "ftp",
    "ftps",
    "ftpes",
    "sftp",
    "scp",
    "ssh",
    "dav",
    "davs",
    "s3",
    "webcals",
    "sms",
    "mms",
    "mapi",
    "wifi",
    "lpa",
    "nms",
    "magnet",
    "calculator",
    "zune",
    "git-client",
    "tbauth",
    "x-cmdpal",
    "feedback-hub",
    "insiderhub",
];
/// Largo mínimo del sufijo numérico que delata un id interno (`discord-548703355543748619`).
const NUMERIC_ID_LEN: usize = 6;

/// Protocolos que vale la pena mostrar como apps instaladas en "otros protocolos".
pub fn is_listable(scheme: &str) -> bool {
    let s = scheme.to_ascii_lowercase();
    !(s.contains('.')
        || is_system_scheme(&s)
        || NOISE_SCHEMES.contains(&s.as_str())
        || NOISE_PREFIXES.iter().any(|p| s.starts_with(p))
        || s.rsplit_once('-').is_some_and(|(_, tail)| {
            tail.len() >= NUMERIC_ID_LEN && tail.bytes().all(|b| b.is_ascii_digit())
        }))
}

#[cfg(test)]
#[path = "protocols_tests.rs"]
mod tests;
