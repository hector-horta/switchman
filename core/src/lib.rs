//! Núcleo de switchman: limpieza de rastreo, reglas por dominio y lanzamiento de navegadores.
pub mod config;
pub mod config_types;
pub mod deeplink_apps;
pub mod deeplinks;
pub mod detector;
pub mod error;
pub mod launcher;
pub mod matcher;
pub mod paths;
#[cfg(windows)]
pub mod protocols;
#[cfg(windows)]
pub mod registry;
pub mod revision;
pub mod router;
pub mod sanitizer;
pub mod store;
#[cfg(windows)]
pub mod webapps;
