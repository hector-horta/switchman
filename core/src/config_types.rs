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
        }
    }
}
