//! Reglas y configuración raíz.
use crate::config::Browser;
use serde::{Deserialize, Serialize};

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub pattern: String,
    pub browser_id: String,
    #[serde(default = "yes")]
    pub strip_tracking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub default_browser_id: String,
    #[serde(default = "yes")]
    pub strip_tracking: bool,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub custom_browsers: Vec<Browser>,
    #[serde(default)]
    pub extra_tracking_params: Vec<String>,
    /// Ids del catálogo de deep links activados (WhatsApp, Discord…).
    #[serde(default)]
    pub app_links: Vec<String>,
    /// Deep links definidos por el usuario para apps que el catálogo no conoce.
    #[serde(default)]
    pub custom_deeplinks: Vec<CustomDeepLink>,
    /// Redirecciones, antes de las reglas. Sin el campo (config anterior) se usan las de fábrica.
    #[serde(default = "default_redirects")]
    pub redirects: Vec<Redirect>,
}

/// Redirección: los enlaces que coinciden con `from` (dominio y ruta opcional, como una regla) se abren
/// como `to`, que es otro enlace `http(s)` o la URI de una app (`whatsapp://`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Redirect {
    pub id: String,
    pub from: String,
    pub to: String,
}

/// Redirecciones de fábrica: la página "descarga la app" de Reddit (en el escritorio no sirve y
/// termina en un artículo de ayuda) lleva a la portada. Se puede editar o borrar.
pub fn default_redirects() -> Vec<Redirect> {
    vec![Redirect {
        id: "rd1".into(),
        from: "reddit.com/mobile/download".into(),
        to: "https://www.reddit.com/".into(),
    }]
}

/// Deep link propio: enlaces de `domains` → `template` (`{url}`, `{host}`, `{path}`, `{query}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomDeepLink {
    pub id: String,
    pub name: String,
    pub scheme: String,
    pub domains: Vec<String>,
    pub template: String,
}

/// Igual que un `config.json` vacío: la limpieza de rastreo viene activada.
impl Default for Config {
    fn default() -> Self {
        Self {
            default_browser_id: String::new(),
            strip_tracking: yes(),
            rules: Vec::new(),
            custom_browsers: Vec::new(),
            extra_tracking_params: Vec::new(),
            app_links: Vec::new(),
            custom_deeplinks: Vec::new(),
            redirects: default_redirects(),
        }
    }
}
