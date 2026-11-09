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
}

/// Convierte una cadena de enlace en una URL válida con validaciones básicas.
pub fn parse_link(raw: &str) -> Result<Url> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > MAX_URL_LEN {
        return Err(LinkError::InvalidUrl(format!(
            "largo inválido: {}",
            raw.len()
        )));
    }
    let url = Url::parse(raw).map_err(|e| LinkError::InvalidUrl(e.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(LinkError::UnsupportedScheme(url.scheme().to_string()));
    }
    if url.host_str().is_none() {
        return Err(LinkError::InvalidUrl("sin host".into()));
    }
    Ok(url)
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
    match loaded {
        Loaded::Fallback(browser) => {
            let stripped = strip_tracking(&mut url, &[]);
            Ok(Decision {
                browser: browser.clone(),
                url,
                rule_id: None,
                stripped,
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
            })
        }
    }
}

#[cfg(test)]
#[path = "router_tests.rs"]
mod tests;
