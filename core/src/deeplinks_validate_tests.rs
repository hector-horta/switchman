use super::*;

fn link(scheme: &str, domains: &[&str], template: &str) -> CustomDeepLink {
    CustomDeepLink {
        id: "steam".into(),
        name: "Steam".into(),
        scheme: scheme.into(),
        domains: domains.iter().map(|d| d.to_string()).collect(),
        template: template.into(),
    }
}

fn config_with(app_links: &[&str], custom: Vec<CustomDeepLink>) -> Config {
    Config {
        default_browser_id: "edge".into(),
        app_links: app_links.iter().map(|a| a.to_string()).collect(),
        custom_deeplinks: custom,
        ..Config::default()
    }
}

#[test]
fn known_app_links_are_valid() {
    assert!(validate(&config_with(&["whatsapp", "discord"], vec![])).is_ok());
    assert!(validate(&config_with(&["myspace"], vec![])).is_err());
}

#[test]
fn custom_link_is_valid() {
    assert!(validate(&config_with(
        &[],
        vec![link(
            "steam",
            &["store.steampowered.com"],
            "steam://openurl/{url}"
        )]
    ))
    .is_ok());
}

#[test]
fn custom_link_rejects_bad_scheme() {
    assert!(validate(&config_with(
        &[],
        vec![link("https", &["a.com"], "https://{path}")]
    ))
    .is_err());
    assert!(validate(&config_with(
        &[],
        vec![link("file", &["a.com"], "file:///{path}")]
    ))
    .is_err());
    assert!(validate(&config_with(
        &[],
        vec![link("Steam App", &["a.com"], "Steam App://x")]
    ))
    .is_err());
    assert!(validate(&config_with(&[], vec![link("", &["a.com"], "://x")])).is_err());
}

#[test]
fn custom_link_template_must_use_its_scheme() {
    assert!(validate(&config_with(
        &[],
        vec![link("steam", &["a.com"], "discord://x")]
    ))
    .is_err());
}

#[test]
fn custom_link_needs_valid_domains() {
    assert!(validate(&config_with(&[], vec![link("steam", &[], "steam://x")])).is_err());
    assert!(validate(&config_with(
        &[],
        vec![link("steam", &["a.com/path"], "steam://x")]
    ))
    .is_err());
}

#[test]
fn custom_link_needs_a_name_and_valid_id() {
    let mut l = link("steam", &["a.com"], "steam://x");
    l.name = "  ".into();
    assert!(validate(&config_with(&[], vec![l])).is_err());

    let mut l = link("steam", &["a.com"], "steam://x");
    l.id = "no valido".into();
    assert!(validate(&config_with(&[], vec![l])).is_err());
}

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

#[test]
fn target_needs_the_app_enabled_and_installed() {
    let installed = |scheme: &str| scheme == "whatsapp";
    let on = config_with(&["whatsapp"], vec![]);
    let off = config_with(&[], vec![]);
    let hit = target(&on, &url("https://wa.me/56912345678"), &installed).unwrap();
    assert_eq!(hit.id, "whatsapp");
    assert_eq!(hit.uri, "whatsapp://send?phone=56912345678");
    assert!(target(&off, &url("https://wa.me/56912345678"), &installed).is_none());
    assert!(target(&on, &url("https://wa.me/56912345678"), &|_: &str| false).is_none());
}

#[test]
fn target_falls_back_when_the_link_has_no_app_form() {
    let on = config_with(&["whatsapp"], vec![]);
    assert!(target(&on, &url("https://wa.me/message/ABC"), &|_: &str| true).is_none());
    assert!(target(&on, &url("https://example.com/"), &|_: &str| true).is_none());
}

#[test]
fn custom_deeplink_wins_over_catalog() {
    let custom = link(
        "steam",
        &["store.steampowered.com"],
        "steam://openurl/{url}",
    );
    let cfg = config_with(&[], vec![custom]);
    let hit = target(
        &cfg,
        &url("https://store.steampowered.com/app/10/"),
        &|s: &str| s == "steam",
    )
    .unwrap();
    assert_eq!(hit.name, "Steam");
    assert_eq!(
        hit.uri,
        "steam://openurl/https://store.steampowered.com/app/10/"
    );
    assert!(target(
        &cfg,
        &url("https://store.steampowered.com/app/10/"),
        &|_: &str| false
    )
    .is_none());
}
