//! Traductores del catálogo de deep links: enlace web → URI del protocolo de la app.
//! Formatos públicos de cada servicio; ante cualquier forma desconocida, `None` (navegador).
use crate::deeplinks::enc;
use url::Url;

/// Segmentos no vacíos del path.
fn segments(url: &Url) -> Vec<&str> {
    url.path_segments()
        .map(|s| s.filter(|x| !x.is_empty()).collect())
        .unwrap_or_default()
}

/// Valor decodificado de un parámetro del query.
fn param(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

/// WhatsApp: `wa.me/<número>` y `api.whatsapp.com/send` → chat; `chat.whatsapp.com/<código>` → grupo.
/// `wa.me/message/…` (WhatsApp Business) no trae el número: `None`.
pub fn whatsapp(url: &Url) -> Option<String> {
    let host = url.host_str()?.to_ascii_lowercase();
    let segs = segments(url);
    let phone = match host.as_str() {
        "chat.whatsapp.com" => return Some(format!("whatsapp://chat?code={}", enc(segs.first()?))),
        "web.whatsapp.com" => return Some("whatsapp://".to_string()),
        "wa.me" => match segs.first()? {
            &"message" => return None,
            first => Some((*first).to_string()),
        },
        "api.whatsapp.com" if segs == ["send"] => param(url, "phone"),
        _ => return None,
    };
    if phone
        .as_deref()
        .is_some_and(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    let mut parts: Vec<String> = phone.iter().map(|p| format!("phone={}", enc(p))).collect();
    parts.extend(param(url, "text").map(|t| format!("text={}", enc(&t))));
    Some(if parts.is_empty() {
        "whatsapp://send".to_string()
    } else {
        format!("whatsapp://send?{}", parts.join("&"))
    })
}

/// Convierte un deep link de Discord a una URI válida.
pub fn discord(url: &Url) -> Option<String> {
    let host = url.host_str()?.to_ascii_lowercase();
    let segs = segments(url);

    if host == "discord.gg" {
        let code = segs.first()?;
        Some(format!("discord://-/invite/{code}"))
    } else if host.ends_with("discord.com") || host.ends_with("discordapp.com") {
        match *segs.first()? {
            "invite" => {
                let code = segs.get(1)?;
                Some(format!("discord://-/invite/{code}"))
            }
            "channels" if segs.len() > 1 => Some(format!("discord://-/{}", segs.join("/"))),
            _ => None,
        }
    } else {
        None
    }
}

/// Teams: enlaces `/l/` de reuniones y chats.
pub fn teams(url: &Url) -> Option<String> {
    if !url.path().starts_with("/l/") {
        return None;
    }
    let mut out = format!("msteams:{}", url.path());
    if let Some(q) = url.query() {
        out.push('?');
        out.push_str(q);
    }
    Some(out)
}

/// Convierte un deep link de Telegram a una URI válida.
pub fn telegram(url: &Url) -> Option<String> {
    let segs = segments(url);
    if let Some(first) = segs.first() {
        if let Some(invite) = first.strip_prefix('+') {
            return if !invite.is_empty() {
                Some(format!("tg://join?invite={}", enc(invite)))
            } else {
                None
            };
        }
        if *first == "joinchat" {
            let invite = segs.get(1)?;
            return Some(format!("tg://join?invite={}", enc(invite)));
        }
        if first.is_empty() || !first.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return None;
        }
        let reserved = ["share", "s", "addstickers", "proxy", "socks"].contains(first);
        if reserved {
            return None;
        }

        let mut out = format!("tg://resolve?domain={}", first);
        if let Some(post) = segs.get(1) {
            if !post.is_empty() && post.chars().all(|c| c.is_ascii_digit()) {
                out.push_str(&format!("&post={}", enc(post)));
            }
        }
        return Some(out);
    }
    None
}

/// Zoom: enlaces `/j/<id>` de reunión, con contraseña si viene.
pub fn zoom(url: &Url) -> Option<String> {
    let segs = segments(url);
    if segs.first() != Some(&"j") {
        return None;
    }
    let id = segs.get(1)?;
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let mut out = format!("zoommtg://zoom.us/join?action=join&confno={}", id);
    if let Some(pwd) = param(url, "pwd") {
        out.push_str(&format!("&pwd={}", enc(&pwd)));
    }
    Some(out)
}

/// Convierte un deep link de Slack a una URI válida.
pub fn slack(url: &Url) -> Option<String> {
    let host = url.host_str()?.to_ascii_lowercase();
    if host != "app.slack.com" {
        return None;
    }
    let segs = segments(url);
    if segs.first() != Some(&"client") {
        return None;
    }
    let team = segs.get(1)?;
    match segs.get(2) {
        Some(channel) => Some(format!(
            "slack://channel?team={}&id={}",
            enc(team),
            enc(channel)
        )),
        None => Some(format!("slack://open?team={}", enc(team))),
    }
}

/// Convierte un deep link de Spotify a una URI válida.
pub fn spotify(url: &Url) -> Option<String> {
    let segs = segments(url);
    let rest: Vec<&str> = segs
        .iter()
        .copied()
        .skip_while(|s| s.starts_with("intl-"))
        .collect();
    let kind = *rest.first()?;
    let id = *rest.get(1)?;
    if !["track", "album", "artist", "playlist", "episode", "show"].contains(&kind) {
        return None;
    }
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(format!("spotify:{kind}:{id}"))
}

/// Convierte un deep link de Notion a una URI válida.
pub fn notion(url: &Url) -> Option<String> {
    let mut out = format!("notion://www.notion.so{}", url.path());
    if let Some(q) = url.query() {
        out.push('?');
        out.push_str(q);
    }
    Some(out)
}

#[cfg(test)]
#[path = "deeplink_apps_tests.rs"]
mod tests;
