//! `apps`: catálogo de deep links con su estado (instalada, activada), los deep links propios y los
//! demás protocolos instalados que el catálogo no conoce (para crear deep links propios).
//! `check-deeplink` y `open-uri` sirven al editor de deep links propios (vista previa y Probar).
use crate::config_cmds::{print_json, read_stdin_json};
use serde_json::{json, Value};
use switchman_core::config;
use switchman_core::config_types::CustomDeepLink;
use switchman_core::error::{LinkError, Result};
use switchman_core::{deeplink_check, deeplinks, paths, protocols};
use tracing::info;

/// Devuelve un catálogo de deep links con su estado (instalada, activado), los deep links propios y protocolos instalados no conocidos por el catálogo.
pub fn apps() -> Result<()> {
    let config = config::load(&paths::config_path()?).unwrap_or_default();
    let catalog: Vec<Value> = deeplinks::CATALOG
        .iter()
        .map(|app| {
            json!({
                "id": app.id,
                "name": app.name,
                "scheme": app.scheme,
                "domains": app.domains,
                "installed": protocols::scheme_registered(app.scheme),
                "enabled": config.app_links.iter().any(|id| id == app.id)
            })
        })
        .collect();
    let custom: Vec<Value> = config
        .custom_deeplinks
        .iter()
        .map(|link| {
            json!({
                "id": link.id,
                "name": link.name,
                "scheme": link.scheme,
                "domains": link.domains,
                "template": link.template,
                "installed": protocols::scheme_registered(&link.scheme)
            })
        })
        .collect();
    let known: Vec<String> = deeplinks::CATALOG
        .iter()
        .map(|app| app.scheme.to_string())
        .chain(
            config
                .custom_deeplinks
                .iter()
                .map(|link| link.scheme.to_ascii_lowercase()),
        )
        .collect();
    let other: Vec<String> = protocols::registered_schemes()
        .into_iter()
        .filter(|scheme| protocols::is_listable(scheme) && !known.contains(scheme))
        .collect();
    let text = serde_json::to_string(&json!({
        "catalog": catalog,
        "custom": custom,
        "other_schemes": other
    }))
    .map_err(|e| LinkError::Config(e.to_string()))?;
    println!("{}", text);
    Ok(())
}

/// `check-deeplink [--url EJEMPLO]`: revisa campo por campo el deep link propio que llega por stdin
/// (JSON) y, con un enlace de ejemplo, en qué URI se convierte. No guarda nada.
pub fn check_deeplink(example: Option<&str>) -> Result<()> {
    let link: CustomDeepLink = read_stdin_json()?;
    print_json(&json!({
        "issues": deeplink_check::issues(&link),
        "installed": protocols::scheme_registered(link.scheme.trim()),
        "preview": example.map(|e| deeplink_check::preview(&link, e)),
    }))
}

/// `open-uri URI`: abre la URI en su app (botón Probar). Solo protocolos registrados que no sean
/// del sistema: nunca abre páginas web ni archivos.
pub fn open_uri(uri: &str) -> Result<()> {
    let scheme = uri.split_once(':').map_or("", |(s, _)| s);
    if !protocols::scheme_registered(scheme) {
        return Err(LinkError::UnsupportedScheme(scheme.to_string()));
    }
    protocols::open_uri(uri)?;
    info!(%scheme, "deep link tried from the editor");
    Ok(())
}
