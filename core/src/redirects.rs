//! Redirecciones configurables: un enlace que coincide con `from` se abre como `to`, una sola vez y
//! antes de las reglas. `to` es otro enlace `http(s)` (después siguen reglas y limpieza) o la URI de
//! una app, que se abre como un deep link (si la app no responde, el enlace original va al navegador).
use crate::config::check_id;
use crate::config_types::{Config, Redirect};
use crate::deeplink_check::valid_scheme;
use crate::error::{LinkError, Result};
use crate::matcher::HostPattern;
use crate::router::parse_link;
use url::Url;

/// Límites (frontera: vienen de config.json).
const MAX_REDIRECTS: usize = 100;
const MAX_TARGET: usize = 2048;

/// A dónde lleva una redirección que coincidió.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Otro enlace web: sigue por reglas y limpieza de rastreo.
    Link(Url),
    /// URI de una app (`whatsapp://`).
    App(String),
}

/// Destino de `to`: enlace `http(s)` si lo es, o URI de app si su protocolo es válido.
fn target_of(redirect: &Redirect) -> Option<Target> {
    let to = redirect.to.trim();
    if let Ok(url) = parse_link(to) {
        return Some(Target::Link(url));
    }
    let (scheme, _) = to.split_once(':')?;
    let scheme = scheme.to_ascii_lowercase();
    (to.len() <= MAX_TARGET && valid_scheme(&scheme)).then(|| Target::App(to.to_string()))
}

/// Como una regla, salvo que `dominio/` (barra final sin ruta) es solo la portada, no todo el sitio.
fn matches(from: &str, url: &Url) -> bool {
    let Ok(pattern) = HostPattern::parse(from) else {
        return false;
    };
    let bare = from.trim();
    let bare = bare
        .strip_prefix("https://")
        .or_else(|| bare.strip_prefix("http://"))
        .unwrap_or(bare);
    let home_only = bare.ends_with('/') && bare.matches('/').count() == 1;
    pattern.matches(url) && (!home_only || url.path() == "/")
}

/// Destino de la primera redirección que coincide con el enlace (las inválidas se saltan).
pub fn apply(config: &Config, url: &Url) -> Option<Target> {
    config
        .redirects
        .iter()
        .filter(|r| matches(&r.from, url))
        .find_map(target_of)
}

/// Valida las redirecciones de config.json (id, origen como el de una regla y destino utilizable).
pub fn validate(config: &Config) -> Result<()> {
    if config.redirects.len() > MAX_REDIRECTS {
        return Err(LinkError::Config(format!(
            "at most {MAX_REDIRECTS} redirects"
        )));
    }

    for r in &config.redirects {
        check_id("redirect", &r.id)?;
        HostPattern::parse(&r.from)?;
        if target_of(r).is_none() {
            return Err(LinkError::Config(format!(
                "redirect {:?}: the target must be an http(s) link or an app URI",
                r.from
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "redirects_tests.rs"]
mod tests;
