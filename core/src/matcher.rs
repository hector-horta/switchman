//! Coincidencia de reglas por dominio (con subdominios) y prefijo de ruta opcional.
use crate::error::{LinkError, Result};
use url::Url;

/// Patrón normalizado: dominio en minúsculas/punycode y prefijo de ruta opcional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostPattern {
    domain: String,
    path_prefix: Option<String>,
}

impl HostPattern {
    /// Normaliza un patrón de regla: `github.com`, `*.github.com`, `https://github.com`
    /// o `github.com/rust-lang/*` (dominio + prefijo de ruta).
    pub fn parse(pattern: &str) -> Result<Self> {
        let mut rest = pattern.trim();
        for scheme in ["https://", "http://"] {
            if let Some(r) = rest.strip_prefix(scheme) {
                rest = r;
            }
        }
        rest = rest.strip_prefix("*.").unwrap_or(rest);

        let (host, path) = rest
            .split_once('/')
            .map_or((rest, String::new()), |(h, p)| (h, format!("/{p}")));

        if host.is_empty() || host.contains('*') || host.contains(char::is_whitespace) {
            return Err(LinkError::InvalidPattern(pattern.to_string()));
        }

        let parsed = Url::parse(&format!("http://{host}/"))
            .map_err(|_| LinkError::InvalidPattern(pattern.to_string()))?;
        let domain = parsed
            .host_str()
            .ok_or_else(|| LinkError::InvalidPattern(pattern.to_string()))?
            .to_string();
        if parsed.port().is_some() {
            return Err(LinkError::InvalidPattern(pattern.to_string()));
        }

        let p = path.trim_end_matches('*').trim_end_matches('/');
        let path_prefix = if p.is_empty() {
            None
        } else {
            Some(p.to_string())
        };

        Ok(Self {
            domain,
            path_prefix,
        })
    }

    /// Devuelve true si la URL coincide con este patrón de dominio y prefijo de ruta.
    pub fn matches(&self, url: &Url) -> bool {
        let Some(host) = url.host_str() else {
            return false;
        };
        let host_ok = host == self.domain || host.ends_with(&format!(".{}", self.domain));
        if !host_ok {
            return false;
        }

        match &self.path_prefix {
            None => true,
            Some(prefix) => {
                let path = url.path();
                path == prefix || path.starts_with(&format!("{prefix}/"))
            }
        }
    }
}

#[cfg(test)]
#[path = "matcher_tests.rs"]
mod tests;
