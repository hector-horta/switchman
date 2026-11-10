//! Subcomandos del CLI. Toda la lógica vive en `switchman_core`.
use serde_json::json;
use switchman_core::config::{self, Browser, Loaded};
use switchman_core::error::{LinkError, Result};
use switchman_core::{detector, launcher, paths, registry, router};
use tracing::{info, warn};

/// URI de Configuración de Windows para elegir el navegador predeterminado.
const SETTINGS_URI: &str = "ms-settings:defaultapps?registeredAppUser=Switchman";

/// Navegadores disponibles: personalizados, rutas conocidas y registrados en Windows (sin duplicados).
fn all_browsers(loaded: &Loaded) -> Vec<Browser> {
    let custom = match loaded {
        Loaded::Config(c) => c.custom_browsers.clone(),
        Loaded::Fallback(b) => vec![b.clone()],
    };
    let env = |name: &str| std::env::var(name).ok();
    let exists = |p: &std::path::Path| p.is_file();
    let known = detector::detect_known(&env, &exists);
    detector::merge(vec![custom, known, registry::installed_browsers()])
}

/// Config del usuario; si falta o está corrupta, una vacía (solo para listar navegadores).
fn config_or_default() -> Result<Loaded> {
    let config = config::load(&paths::config_path()?).unwrap_or_else(|e| {
        warn!(error = %e, "config no disponible; uso una vacía");
        Default::default()
    });
    Ok(Loaded::Config(config))
}

/// Decide navegador y URL; si falta la config usa el respaldo, y si falta este, el primer detectado.
fn route(raw: &str) -> Result<router::Decision> {
    let loaded =
        config::load_effective(&paths::config_path()?, &paths::fallback_path()?).or_else(|e| {
            warn!(error = %e, "sin config ni respaldo; uso el primer navegador detectado");
            let first = all_browsers(&Loaded::Config(Default::default()))
                .into_iter()
                .next()
                .ok_or(LinkError::BrowserNotFound("ninguno".into()))?;
            Ok::<Loaded, LinkError>(Loaded::Fallback(first))
        })?;

    let browsers = all_browsers(&loaded);
    let decision = match router::decide(&loaded, &browsers, raw) {
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

/// Enruta y abre un enlace (lo invoca Windows en cada clic).
pub fn open_url(raw: &str) -> Result<()> {
    let decision = route(raw)?;
    let self_exe = std::env::current_exe()?;
    launcher::launch(&decision.browser, &decision.url, &self_exe)?;
    info!(browser = %decision.browser.id, rule = ?decision.rule_id, stripped = decision.stripped, "enlace abierto");
    Ok(())
}

/// Prueba la ruta de un enlace y devuelve los detalles del navegador seleccionado como JSON.
pub fn test_url(raw: &str) -> Result<()> {
    let decision = route(raw)?;
    let out = json!({ "browser_id": decision.browser.id, "browser_name": decision.browser.name, "url": decision.url.as_str(), "rule_id": decision.rule_id, "stripped": decision.stripped });
    println!("{out}");
    Ok(())
}

/// Imprime todos los navegadores detectados como un array JSON de objetos con sus IDs y nombres.
pub fn detect() -> Result<()> {
    let browsers = all_browsers(&config_or_default()?);
    let out = serde_json::to_string(&browsers).map_err(|e| LinkError::Config(e.to_string()))?;
    println!("{out}");
    Ok(())
}

/// Establece un navegador como respaldo para cuando no se encuentre la configuración principal.
pub fn set_fallback(id: &str) -> Result<()> {
    let browser = all_browsers(&config_or_default()?)
        .into_iter()
        .find(|b| b.id.eq_ignore_ascii_case(id.trim()))
        .ok_or_else(|| LinkError::BrowserNotFound(id.to_string()))?;
    config::save_fallback(&paths::fallback_path()?, &browser)?;
    info!(browser = %browser.id, "navegador de respaldo guardado");
    Ok(())
}

/// Registra este ejecutable como navegador predeterminado en Windows y abre la configuración del sistema.
pub fn register() -> Result<()> {
    let exe = std::env::current_exe()?;
    registry::register(&exe)?;
    std::process::Command::new("explorer.exe")
        .arg(SETTINGS_URI)
        .spawn()?;
    info!(exe = %exe.display(), "switchman registrado");
    Ok(())
}

/// Elimina el registro de switchman en HKCU.
pub fn unregister() -> Result<()> {
    registry::unregister()?;
    info!("switchman desregistrado");
    Ok(())
}
