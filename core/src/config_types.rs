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
        }
    }
}
