use super::*;

fn tr(id: &str, link: &str) -> Option<String> {
    let app = CATALOG.iter().find(|a| a.id == id).unwrap();
    app.translate(&Url::parse(link).unwrap())
}

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

#[test]
fn find_app_by_domain_and_subdomain() {
    assert_eq!(
        find_app(&Url::parse("https://api.whatsapp.com/send?phone=1").unwrap()).map(|a| a.id),
        Some("whatsapp")
    );
    assert_eq!(
        find_app(&Url::parse("https://discord.gg/x").unwrap()).map(|a| a.id),
        Some("discord")
    );
    assert!(find_app(&Url::parse("https://notwhatsapp.com/").unwrap()).is_none());
    assert!(find_app(&Url::parse("https://example.com/").unwrap()).is_none());
}

#[test]
fn template_placeholders() {
    let u = Url::parse("https://a.com/p/q?x=1").unwrap();
    assert_eq!(
        apply_template("steam://openurl/{url}", &u),
        "steam://openurl/https://a.com/p/q?x=1"
    );
    assert_eq!(
        apply_template("myapp://{host}/{path}?{query}", &u),
        "myapp://a.com/p/q?x=1"
    );
}

#[test]
fn whatsapp_links() {
    assert_eq!(
        tr("whatsapp", "https://wa.me/56912345678?text=Hola%20mundo"),
        s("whatsapp://send?phone=56912345678&text=Hola%20mundo")
    );
    assert_eq!(
        tr("whatsapp", "https://wa.me/56912345678"),
        s("whatsapp://send?phone=56912345678")
    );
    assert_eq!(tr("whatsapp", "https://wa.me/message/ABC123"), None);
    assert_eq!(
        tr(
            "whatsapp",
            "https://api.whatsapp.com/send?phone=56912345678&text=Hi"
        ),
        s("whatsapp://send?phone=56912345678&text=Hi")
    );
    assert_eq!(
        tr("whatsapp", "https://chat.whatsapp.com/AbCdEf123"),
        s("whatsapp://chat?code=AbCdEf123")
    );
    assert_eq!(
        tr("whatsapp", "https://web.whatsapp.com/"),
        s("whatsapp://")
    );
    assert_eq!(tr("whatsapp", "https://www.whatsapp.com/download"), None);
}

#[test]
fn discord_links() {
    assert_eq!(
        tr("discord", "https://discord.gg/abc123"),
        s("discord://-/invite/abc123")
    );
    assert_eq!(
        tr("discord", "https://discord.com/invite/abc123"),
        s("discord://-/invite/abc123")
    );
    assert_eq!(
        tr("discord", "https://discord.com/channels/111/222"),
        s("discord://-/channels/111/222")
    );
    assert_eq!(tr("discord", "https://discord.com/nitro"), None);
}
