//! Enrutamiento: qué destino abre un enlace. Primero con los destinos baratos y, solo si la regla
//! apunta a otro, con el registro de Windows y las webapps instaladas.
use switchman_core::config::{self, Browser, Loaded};
use switchman_core::error::{LinkError, Result};
use switchman_core::{detector, paths, registry, router, webapps};
use tracing::warn;

/// Destinos baratos de calcular: personalizados y rutas conocidas (sin leer registro ni accesos directos).
fn quick_browsers(loaded: &Loaded) -> Vec<Browser> {
    let custom = match loaded {
        Loaded::Config(c) => c.custom_browsers.clone(),
        Loaded::Fallback(b) => vec![b.clone()],
    };
    let env = |name: &str| std::env::var(name).ok();
    let exists = |p: &std::path::Path| p.is_file();
    detector::merge(vec![custom, detector::detect_known(&env, &exists)])
}

/// Todos los destinos: los baratos, los registrados en Windows y las webapps instaladas (sin duplicados).
pub fn all_browsers(loaded: &Loaded) -> Vec<Browser> {
    detector::merge(vec![
        quick_browsers(loaded),
        registry::installed_browsers(),
        webapps::installed_webapps(),
    ])
}

/// Decide navegador y URL; si falta la config usa el respaldo, y si falta este, el primer detectado.
pub fn route(raw: &str) -> Result<router::Decision> {
    let loaded =
        config::load_effective(&paths::config_path()?, &paths::fallback_path()?).or_else(|e| {
            warn!(error = %e, "sin config ni respaldo; uso el primer navegador detectado");
            let first = all_browsers(&Loaded::Config(Default::default()))
                .into_iter()
                .next()
                .ok_or(LinkError::BrowserNotFound("ninguno".into()))?;
            Ok::<Loaded, LinkError>(Loaded::Fallback(first))
        })?;

    // Casi siempre basta con los destinos baratos. Si la regla apunta a otro (registro o webapp),
    // recién entonces se leen el registro y los accesos directos (~50 ms más por clic).
    let decision = match router::decide(&loaded, &quick_browsers(&loaded), raw) {
        Err(LinkError::BrowserNotFound(_)) => full_route(&loaded, raw)?,
        other => other?,
    };

    Ok(decision)
}

/// Segunda pasada con todos los destinos; si aún falta el navegador, el respaldo o el primero detectado.
fn full_route(loaded: &Loaded, raw: &str) -> Result<router::Decision> {
    let browsers = all_browsers(loaded);
    let decision = match router::decide(loaded, &browsers, raw) {
        Err(LinkError::BrowserNotFound(id)) => {
            warn!(%id, "navegador de la regla no encontrado; uso el respaldo");
            let fallback = switchman_core::store::read_json::<Browser>(&paths::fallback_path()?)
                .ok()
                .or_else(|| browsers.first().cloned())
                .ok_or(LinkError::BrowserNotFound(id))?;
            router::decide(&Loaded::Fallback(fallback), &browsers, raw)?
        }
        other => other?,
    };

    Ok(decision)
}
