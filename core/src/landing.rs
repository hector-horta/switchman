//! Páginas de "descarga la app" que Phone Link abre al tocar la notificación de una app que no existe
//! en Windows (Reddit manda `reddit.com/mobile/download`, que termina en un artículo de ayuda). Se
//! cambian por la portada del servicio, que es lo útil en el escritorio; después siguen las reglas.
use crate::deeplinks::host_matches;
use url::Url;

/// (dominio, ruta, portada). El dominio incluye subdominios; la ruta se compara sin `/` final.
const LANDINGS: &[(&str, &str, &str)] =
    &[("reddit.com", "/mobile/download", "https://www.reddit.com/")];

/// Portada del servicio si `url` es una de sus páginas de "descarga la app".
pub fn rewrite(url: &Url) -> Option<Url> {
    let host = url.host_str()?;
    let path = url.path().trim_end_matches('/');
    let (_, _, home) = LANDINGS
        .iter()
        .find(|(domain, page, _)| host_matches(host, domain) && path.eq_ignore_ascii_case(page))?;
    Url::parse(home).ok()
}

#[cfg(test)]
#[path = "landing_tests.rs"]
mod tests;
