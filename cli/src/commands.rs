//! Subcomandos del CLI. Toda la lógica vive en `switchman_core`.
use crate::routing::{all_browsers, route};
use serde_json::json;
use switchman_core::config::{self, Loaded};
use switchman_core::error::{LinkError, Result};
use switchman_core::{launcher, paths, registry};
use tracing::{info, warn};

/// URI de Configuración de Windows para elegir el navegador predeterminado.
const SETTINGS_URI: &str = "ms-settings:defaultapps?registeredAppUser=Switchman";

/// Config del usuario; si falta o está corrupta, una vacía (solo para listar navegadores).
fn config_or_default() -> Result<Loaded> {
    let config = config::load(&paths::config_path()?).unwrap_or_else(|e| {
        warn!(error = %e, "config no disponible; uso una vacía");
        Default::default()
    });
    Ok(Loaded::Config(config))
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
    info!(exe = %exe.display(), "switchman registrado");
    open_settings()
}

/// Abre la Configuración de Windows en la ficha de Switchman con `ShellExecuteW`.
/// `explorer.exe <uri>` no entiende el parámetro `?registeredAppUser=` y abre el Explorador de archivos.
fn open_settings() -> Result<()> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let wide = |s: &str| {
        s.encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>()
    };
    let (verb, uri) = (wide("open"), wide(SETTINGS_URI));
    // SAFETY: cadenas UTF-16 terminadas en 0 que viven durante toda la llamada; hwnd nulo es válido.
    let code = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            uri.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;
    // ShellExecuteW devuelve un valor > 32 si tuvo éxito.
    if code > 32 {
        Ok(())
    } else {
        Err(LinkError::Io(std::io::Error::other(format!(
            "no se pudo abrir Configuración (ShellExecuteW = {code})"
        ))))
    }
}

/// Elimina el registro de switchman en HKCU.
pub fn unregister() -> Result<()> {
    registry::unregister()?;
    info!("switchman desregistrado");
    Ok(())
}
