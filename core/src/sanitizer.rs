//! Limpieza de parámetros de rastreo en URLs http/https.
use url::Url;

/// Parámetros de rastreo exactos (se comparan en minúsculas).
const TRACKING_PARAMS: &[&str] = &[
    "fbclid",
    "igshid",
    "igsh", // Instagram reemplazó igshid por igsh en los enlaces compartidos
    "gclid",
    "gclsrc",
    "dclid",
    "gbraid",
    "wbraid",
    "srsltid",
    "msclkid",
    "twclid",
    "ttclid",
    "li_fat_id",
    "yclid",
    "mc_cid",
    "mc_eid",
    "_hsenc",
    "_hsmi",
    "mkt_tok",
    "vero_id",
    "wickedid",
    "oly_anon_id",
    "oly_enc_id",
    "rb_clickid",
    "s_cid",
    "si",
];

/// Prefijos de rastreo (p. ej. `utm_source`).
const TRACKING_PREFIXES: &[&str] = &["utm_"];

/// Verifica si la clave de parámetro es un parámetro de rastreo.
pub fn is_tracking_param(key: &str, extra: &[String]) -> bool {
    let k = key.trim().to_ascii_lowercase();
    if k.is_empty() {
        return false;
    }
    TRACKING_PARAMS.contains(&k.as_str())
        || TRACKING_PREFIXES
            .iter()
            .any(|p| k.starts_with(p) && k.len() > p.len())
        || extra.iter().any(|e| e.trim().eq_ignore_ascii_case(&k))
}

/// Elimina los parámetros de rastreo del query sin re-codificar los pares conservados.
/// Devuelve `true` si quitó al menos un parámetro.
pub fn strip_tracking(url: &mut Url, extra: &[String]) -> bool {
    let Some(query) = url.query() else {
        return false;
    };
    let query = query.to_string();
    let kept: Vec<&str> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .filter(|piece| {
            let key = piece.split('=').next().unwrap_or("");
            !is_tracking_param(key, extra)
        })
        .collect();

    let removed = kept.len() != query.split('&').filter(|p| !p.is_empty()).count();
    if !removed {
        return false;
    }
    if kept.is_empty() {
        url.set_query(None);
    } else {
        url.set_query(Some(&kept.join("&")));
    }
    true
}

#[cfg(test)]
#[path = "sanitizer_tests.rs"]
mod tests;
