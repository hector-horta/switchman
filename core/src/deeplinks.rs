//! Deep links: abrir enlaces web de apps conocidas en la app de escritorio (WhatsApp, Discord…).
//! Cada app registra un protocolo propio (`whatsapp:`); aquí se traduce el enlace web a ese
//! protocolo. Si un enlace no encaja en la traducción, devuelve `None` y se abre en el navegador.
use crate::config::check_id;
use crate::config_types::{Config, CustomDeepLink};
use crate::deeplink_apps as apps;
use crate::error::{LinkError, Result};
use crate::matcher::HostPattern;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use url::Url;

/// Destino de un deep link: la app y la URI de su protocolo con la que se abre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppTarget {
    pub id: String,
    pub name: String,
    pub uri: String,
}

/// App del catálogo integrado: dominios que cubre, protocolo y su traductor.
pub struct CatalogApp {
    pub id: &'static str,
    pub name: &'static str,
    pub scheme: &'static str,
    pub domains: &'static [&'static str],
    translate: fn(&Url) -> Option<String>,
}

/// Catálogo integrado. Solo apps con formatos de enlace públicos y estables.
pub static CATALOG: &[CatalogApp] = &[
    CatalogApp {
        id: "whatsapp",
        name: "WhatsApp",
        scheme: "whatsapp",
        domains: &["wa.me", "whatsapp.com"],
        translate: apps::whatsapp,
    },
    CatalogApp {
        id: "discord",
        name: "Discord",
        scheme: "discord",
        domains: &["discord.gg", "discord.com", "discordapp.com"],
        translate: apps::discord,
    },
    CatalogApp {
        id: "teams",
        name: "Microsoft Teams",
        scheme: "msteams",
        domains: &["teams.microsoft.com"],
        translate: apps::teams,
    },
    CatalogApp {
        id: "telegram",
        name: "Telegram",
        scheme: "tg",
        domains: &["t.me", "telegram.me"],
        translate: apps::telegram,
    },
    CatalogApp {
        id: "zoom",
        name: "Zoom",
        scheme: "zoommtg",
        domains: &["zoom.us"],
        translate: apps::zoom,
    },
    CatalogApp {
        id: "slack",
        name: "Slack",
        scheme: "slack",
        domains: &["slack.com"],
        translate: apps::slack,
    },
    CatalogApp {
        id: "spotify",
        name: "Spotify",
        scheme: "spotify",
        domains: &["open.spotify.com"],
        translate: apps::spotify,
    },
    CatalogApp {
        id: "notion",
        name: "Notion",
        scheme: "notion",
        domains: &["notion.so"],
        translate: apps::notion,
    },
];

impl CatalogApp {
    /// Traduce el enlace al protocolo de la app; `None` si el enlace no tiene equivalente.
    pub fn translate(&self, url: &Url) -> Option<String> {
        (self.translate)(url)
    }
}

/// Codifica un valor para ponerlo en el query de una URI de app.
pub(crate) fn enc(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

/// El host es el dominio o un subdominio suyo, sin distinguir mayúsculas.
pub fn host_matches(host: &str, domain: &str) -> bool {
    let host = host.to_ascii_lowercase();
    let domain = domain.to_ascii_lowercase();
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// Encuentra la app del catálogo que incluye al dominio del host de la URL.
pub fn find_app(url: &Url) -> Option<&'static CatalogApp> {
    let host = url.host_str()?;
    CATALOG
        .iter()
        .find(|app| app.domains.iter().any(|d| host_matches(host, d)))
}

/// Reemplaza los marcadores en el template con los valores de la URL.
/// {url}: valor completo de la URL (ej: "https://wa.me/123").
/// {host}: dominio del host (ej: "wa.me" o "discord.gg").
/// {path}: parte de la ruta después de "/", sin el prefijo inicial.
/// {query}: parámetros de consulta sin el `?` (ej: "token=abc"); vacío si no hay.
pub fn apply_template(template: &str, url: &Url) -> String {
    let url_str = url.as_str();
    let host = url.host_str().unwrap_or("");
    let path = url.path().trim_start_matches('/');
    template
        .replace("{url}", url_str)
        .replace("{host}", host)
        .replace("{path}", path)
        .replace("{query}", url.query().unwrap_or(""))
}

/// App que abre el enlace: deep links propios primero, después el catálogo si está activado e instalado.
pub fn target(config: &Config, url: &Url, registered: &dyn Fn(&str) -> bool) -> Option<AppTarget> {
    let host = url.host_str()?;
    for link in &config.custom_deeplinks {
        if link.domains.iter().any(|d| host_matches(host, d)) && registered(&link.scheme) {
            return Some(AppTarget {
                id: link.id.clone(),
                name: link.name.clone(),
                uri: apply_template(&link.template, url),
            });
        }
    }
    let app = find_app(url)?;
    if !config.app_links.iter().any(|id| id == app.id) || !registered(app.scheme) {
        return None;
    }
    let uri = app.translate(url)?;
    Some(AppTarget {
        id: app.id.to_string(),
        name: app.name.to_string(),
        uri,
    })
}

/// Protocolos que un deep link propio no puede usar: abrirían el navegador o archivos locales.
const FORBIDDEN_SCHEMES: &[&str] = &["http", "https", "file", "javascript", "vbscript", "data"];
/// Límites de un deep link propio (frontera: vienen de config.json).
const MAX_NAME: usize = 64;
const MAX_SCHEME: usize = 32;
const MAX_TEMPLATE: usize = 512;
const MAX_DOMAINS: usize = 20;

/// valida las apps activadas y los deep links propios de config.json.
pub fn validate(config: &Config) -> Result<()> {
    for id in &config.app_links {
        if !CATALOG.iter().any(|app| app.id == id) {
            return Err(LinkError::Config(format!(
                "app de deep link desconocida: {id:?}"
            )));
        }
    }

    for link in &config.custom_deeplinks {
        validate_custom(link)?;
    }

    Ok(())
}

/// Valida un deep link personalizado según las reglas de configuración.
fn validate_custom(link: &CustomDeepLink) -> Result<()> {
    let bad = |why: &str| {
        Err(LinkError::Config(format!(
            "deep link {:?}: {why}",
            link.name
        )))
    };

    check_id("deep link", &link.id)?;

    if link.name.trim().is_empty() || link.name.chars().count() > MAX_NAME {
        return bad("nombre vacío o demasiado largo");
    }

    if !valid_scheme(&link.scheme) {
        return bad("protocolo inválido");
    }

    if link.template.len() > MAX_TEMPLATE
        || !link
            .template
            .to_ascii_lowercase()
            .starts_with(&format!("{}:", link.scheme))
    {
        return bad("la plantilla debe empezar con su protocolo");
    }

    if link.domains.is_empty() || link.domains.len() > MAX_DOMAINS {
        return bad("se necesita entre 1 y 20 dominios");
    }

    for domain in &link.domains {
        if domain.contains('/') {
            return bad("los dominios no llevan ruta");
        }
        HostPattern::parse(domain)?;
    }

    Ok(())
}

/// Verifica si el esquema es válido para un deep link propio.
/// Debe cumplir: no estar vacío, longitud ≤ 32, primera letra minúscula ASCII,
/// todos los caracteres son minúsculas ASCII, dígitos o '+', '-', '.',
/// y no está en la lista de esquemas prohibidos.
fn valid_scheme(scheme: &str) -> bool {
    !scheme.is_empty()
        && scheme.len() <= MAX_SCHEME
        && scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
        && scheme.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '+' || c == '-' || c == '.'
        })
        && !FORBIDDEN_SCHEMES.contains(&scheme)
}

#[cfg(test)]
#[path = "deeplinks_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "deeplinks_validate_tests.rs"]
mod validate_tests;
