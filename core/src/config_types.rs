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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
