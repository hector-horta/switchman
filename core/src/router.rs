//! Decide el navegador y la URL final para un enlace entrante.
use crate::config::{Browser, Loaded};
use crate::config_types::Config;
use crate::error::{LinkError, Result};
use crate::matcher::HostPattern;
use crate::sanitizer::strip_tracking;
use url::Url;

/// Largo máximo aceptado para un enlace.
const MAX_URL_LEN: usize = 8192;

/// Resultado de enrutar un enlace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub browser: Browser,
    pub url: Url,
    pub rule_id: Option<String>,
    pub stripped: bool,
    /// App de escritorio que abre el enlace por deep link (lo completa el CLI); si es `None`, el navegador.
    pub app: Option<crate::deeplinks::AppTarget>,
}

/// Convierte una cadena de enlace en una URL válida con validaciones básicas.
pub fn parse_link(raw: &str) -> Result<Url> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > MAX_URL_LEN {
        return Err(LinkError::InvalidUrl(format!(
            "invalid length: {}",
            raw.len()
        )));
    }
    let url = Url::parse(raw).map_err(|e| LinkError::InvalidUrl(e.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(LinkError::UnsupportedScheme(url.scheme().to_string()));
    }
    if url.host_str().is_none() {
        return Err(LinkError::InvalidUrl("no host".into()));
    }
    Ok(url)
}

/// Archivo local (ruta absoluta o `file:`) que Windows entrega al abrir un `.html` con Switchman:
/// va intacto al navegador por defecto, sin reglas ni limpieza.
pub fn is_local_file(raw: &str) -> bool {
    let raw = raw.trim();
    raw.get(..5)
        .is_some_and(|p| p.eq_ignore_ascii_case("file:"))
        || std::path::Path::new(raw).is_absolute()
}

/// Argumento único que es un enlace http(s) o un archivo local: así llaman a Switchman las apps que
/// lanzan "el navegador" con la URL sola, o la asociación genérica `Applications\switchman-cli.exe`
/// que crea Windows si se elige el .exe a mano (`switchman-cli.exe "%1"`, sin `open-url`).
pub fn is_link_or_file(raw: &str) -> bool {
    let raw = raw.trim();
    let starts = |p: &str| {
        raw.get(..p.len())
            .is_some_and(|h| h.eq_ignore_ascii_case(p))
    };
    starts("http://") || starts("https://") || is_local_file(raw)
}

/// Elige el navegador y la regla de redirección que mejor se ajuste al enlace proporcionado,
/// priorizando patrones válidos en la configuración del usuario.
fn pick(config: &Config, url: &Url) -> (String, Option<String>, bool) {
    for rule in &config.rules {
        let Ok(pattern) = HostPattern::parse(&rule.pattern) else {
            continue;
        };
        if pattern.matches(url) {
            return (
                rule.browser_id.clone(),
                Some(rule.id.clone()),
                rule.strip_tracking,
            );
        }
    }
    (
        config.default_browser_id.clone(),
        None,
        config.strip_tracking,
    )
}

/// Determina el navegador y la URL final para un enlace entrante según la configuración disponible.
pub fn decide(loaded: &Loaded, browsers: &[Browser], raw: &str) -> Result<Decision> {
    let mut url = parse_link(raw)?;
    // Página de "descarga la app" (Phone Link): se abre la portada y las reglas deciden dónde.
    if let Some(home) = crate::landing::rewrite(&url) {
        url = home;
    }
    match loaded {
        Loaded::Fallback(browser) => {
            let stripped = strip_tracking(&mut url, &[]);
            Ok(Decision {
                browser: browser.clone(),
                url,
                rule_id: None,
                stripped,
                app: None,
            })
        }
        Loaded::Config(config) => {
            let (browser_id, rule_id, strip) = pick(config, &url);
            let browser = browsers
                .iter()
                .find(|b| b.id.eq_ignore_ascii_case(&browser_id))
                .cloned()
                .ok_or(LinkError::BrowserNotFound(browser_id))?;
            let stripped = strip && strip_tracking(&mut url, &config.extra_tracking_params);
            Ok(Decision {
                browser,
                url,
                rule_id,
                stripped,
                app: None,
            })
        }
    }
}

#[cfg(test)]
#[path = "router_tests.rs"]
mod tests;
