use super::*;
use crate::deeplinks::apply_template;
use url::Url;

fn link(name: &str, scheme: &str, domains: &[&str], template: &str) -> CustomDeepLink {
    CustomDeepLink {
        id: "dl1".into(),
        name: name.into(),
        scheme: scheme.into(),
        domains: domains.iter().map(|d| d.to_string()).collect(),
        template: template.into(),
    }
}

fn codes(link: &CustomDeepLink) -> Vec<(&'static str, &'static str)> {
    issues(link).iter().map(|i| (i.field, i.code)).collect()
}

#[test]
fn valid_link_has_no_issues() {
    let link = link(
        "Steam",
        "steam",
        &["store.steampowered.com"],
        "steam://openurl/{url}",
    );
    assert!(codes(&link).is_empty());
}

#[test]
fn each_field_reports_its_problem() {
    let link = link("", "Steam App", &[], "x");
    assert_eq!(
        codes(&link),
        vec![
            ("name", "name_invalid"),
            ("scheme", "scheme_invalid"),
            ("template", "template_scheme"),
            ("domains", "domains_count")
        ]
    );
}

#[test]
fn forbidden_scheme_is_invalid() {
    let link = link("A", "https", &["a.com"], "https://{path}");
    assert_eq!(codes(&link), vec![("scheme", "scheme_invalid")]);
}

#[test]
fn domain_with_path_is_reported() {
    let link = link("A", "steam", &["a.com/x"], "steam://{url}");
    assert_eq!(codes(&link), vec![("domains", "domain_path")]);
}

#[test]
fn template_must_start_with_scheme() {
    let link = link("A", "steam", &["a.com"], "other://{url}");
    assert_eq!(codes(&link), vec![("template", "template_scheme")]);
}

#[test]
fn empty_query_drops_the_question_mark() {
    let url = Url::parse("https://a.com/x/y").unwrap();
    assert_eq!(
        apply_template("app://{host}/{path}?{query}", &url),
        "app://a.com/x/y"
    );

    let url = Url::parse("https://a.com/x?k=v").unwrap();
    assert_eq!(
        apply_template("app://{host}/{path}?{query}", &url),
        "app://a.com/x?k=v"
    );
}

#[test]
fn preview_shows_the_uri_and_whether_the_domain_matches() {
    let steam = link(
        "Steam",
        "steam",
        &["steampowered.com"],
        "steam://openurl/{url}",
    );
    let p = preview(&steam, " https://store.steampowered.com/app/10 ");
    assert!(p.valid_url && p.matches);
    assert_eq!(
        p.uri.as_deref(),
        Some("steam://openurl/https://store.steampowered.com/app/10")
    );
    let other = preview(&steam, "https://example.com/");
    assert!(other.valid_url && !other.matches && other.uri.is_some());
    assert!(!preview(&steam, "not a link").valid_url);
    assert!(!preview(&steam, "file:///C:/a.html").valid_url);
}
