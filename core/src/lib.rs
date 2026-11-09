//! Núcleo de ferro-link: limpieza de rastreo, reglas por dominio y lanzamiento de navegadores.
pub mod config;
pub mod config_types;
pub mod detector;
pub mod error;
pub mod launcher;
pub mod matcher;
pub mod paths;
#[cfg(windows)]
pub mod registry;
pub mod router;
pub mod sanitizer;
pub mod store;
