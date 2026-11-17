//! Revisión de un deep link propio campo por campo. La UI muestra cada problema junto a su campo
//! (con su propio texto, según el código) y `deeplinks::validate` usa el primero como error.
use crate::config_types::CustomDeepLink;
use crate::deeplinks::{apply_template, host_matches};
use crate::matcher::HostPattern;
use serde::Serialize;
use url::Url;

/// Protocolos que un deep link propio no puede usar: abrirían el navegador o archivos locales.
const FORBIDDEN_SCHEMES: &[&str] = &["http", "https", "file", "javascript", "vbscript", "data"];
/// Límites de un deep link propio (frontera: vienen de config.json o de la UI).
const MAX_NAME: usize = 64;
const MAX_SCHEME: usize = 32;
const MAX_TEMPLATE: usize = 512;
const MAX_DOMAINS: usize = 20;

/// Problema de un campo: `field` es name, scheme, domains o template; `code` lo identifica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub field: &'static str,
    pub code: &'static str,
}

impl Issue {
    /// Texto en inglés para los errores del CLI (la UI usa el código con sus propios textos).
    pub fn message(&self) -> &'static str {
        match self.code {
            "name_invalid" => "name is empty or too long",
            "scheme_invalid" => "invalid protocol",
            "template_scheme" => "the template must start with its protocol",
            "template_too_long" => "the template is too long",
            "domains_count" => "between 1 and 20 domains are needed",
            "domain_path" => "domains must not include a path",
            _ => "invalid domain",
        }
    }
}

/// Problemas del deep link, como mucho uno por campo y en el orden del formulario.
pub fn issues(link: &CustomDeepLink) -> Vec<Issue> {
    let mut out = Vec::new();

    let name = link.name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        out.push(Issue {
            field: "name",
            code: "name_invalid",
        });
    }

    // Scheme validation
    if !valid_scheme(&link.scheme) {
        out.push(Issue {
            field: "scheme",
            code: "scheme_invalid",
        });
    }

    // Template validation
    if link.template.len() > MAX_TEMPLATE {
        out.push(Issue {
            field: "template",
            code: "template_too_long",
        });
    } else if !link
        .template
        .to_ascii_lowercase()
        .starts_with(&format!("{}:", link.scheme))
    {
        out.push(Issue {
            field: "template",
            code: "template_scheme",
        });
    }

    // Domains validation
    if link.domains.is_empty() || link.domains.len() > MAX_DOMAINS {
        out.push(Issue {
            field: "domains",
            code: "domains_count",
        });
    } else {
        let first_bad_domain = link.domains.iter().find_map(|d| {
            if d.contains('/') {
                Some("domain_path")
            } else if HostPattern::parse(d).is_err() {
                Some("domain_invalid")
            } else {
                None
            }
        });
        if let Some(code) = first_bad_domain {
            out.push(Issue {
                field: "domains",
                code,
            });
        }
    }

    out
}

/// Resultado de probar un enlace de ejemplo: si la plantilla lo captura y en qué URI se convierte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Preview {
    /// El ejemplo es un enlace http(s) válido.
    pub valid_url: bool,
    /// Su dominio está entre los del deep link (si no, Switchman no lo enviaría a la app).
    pub matches: bool,
    /// URI con la que se abriría la app (solo si el enlace es válido).
    pub uri: Option<String>,
}

/// Prueba un enlace de ejemplo con el deep link: si lo captura y en qué URI se convierte.
pub fn preview(link: &CustomDeepLink, example: &str) -> Preview {
    let Ok(url) = Url::parse(example.trim()) else {
        return Preview {
            valid_url: false,
            matches: false,
            uri: None,
        };
    };
    if url.scheme() != "http" && url.scheme() != "https" {
        return Preview {
            valid_url: false,
            matches: false,
            uri: None,
        };
    }
    let Some(host) = url.host_str() else {
        return Preview {
            valid_url: false,
            matches: false,
            uri: None,
        };
    };
    let matches = link.domains.iter().any(|d| host_matches(host, d.trim()));
    Preview {
        valid_url: true,
        matches,
        uri: Some(apply_template(&link.template, &url)),
    }
}

/// Protocolo válido: minúsculas ASCII, dígitos y + - . (empieza con letra), sin los prohibidos.
fn valid_scheme(scheme: &str) -> bool {
    !scheme.is_empty()
        && scheme.len() <= MAX_SCHEME
        && scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
        && scheme.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '+' || c == '-' || c == '.'
        })
        && !FORBIDDEN_SCHEMES.contains(&scheme)
}

#[cfg(test)]
#[path = "deeplink_check_tests.rs"]
mod tests;
