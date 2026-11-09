use super::*;
use crate::config_types::Rule;

fn b(id: &str) -> Browser {
    Browser {
        id: id.into(),
        name: id.into(),
        exe_path: format!("C:/b/{id}.exe"),
        args: vec![],
    }
}

fn browsers() -> Vec<Browser> {
    vec![b("edge"), b("chrome"), b("brave")]
}

fn rule(id: &str, pattern: &str, browser_id: &str, strip_tracking: bool) -> Rule {
    Rule {
        id: id.into(),
        pattern: pattern.into(),
        browser_id: browser_id.into(),
        strip_tracking,
    }
}

fn cfg() -> Loaded {
    Loaded::Config(Config {
        default_browser_id: "edge".into(),
        strip_tracking: true,
        rules: vec![
            rule("gh", "github.com", "chrome", false),
            rule("bv", "brave.com", "brave", true),
        ],
        custom_browsers: vec![],
        extra_tracking_params: vec!["ref".into()],
    })
}

#[test]
fn rejects_non_http_schemes() {
    let schemes = [
        "javascript:alert(1)",
        "file:///C:/x",
        "ftp://a.com/",
        "mailto:a@b.c",
    ];
    for raw in schemes {
        assert!(matches!(
            parse_link(raw),
            Err(LinkError::UnsupportedScheme(_))
        ));
    }
}

#[test]
fn rejects_garbage_and_too_long() {
    assert!(matches!(
        parse_link("not a url"),
        Err(LinkError::InvalidUrl(_))
    ));

    let long = format!("https://a.com/?q={}", "x".repeat(9000));
    assert!(matches!(parse_link(&long), Err(LinkError::InvalidUrl(_))));
}

#[test]
fn accepts_trimmed_https() {
    assert_eq!(
        parse_link("  https://a.com/x  ").unwrap().as_str(),
        "https://a.com/x"
    );
}

#[test]
fn rule_without_strip_keeps_params() {
    let d = decide(&cfg(), &browsers(), "https://github.com/x?utm_source=a").unwrap();
    assert_eq!(d.browser.id, "chrome");
    assert_eq!(d.rule_id.as_deref(), Some("gh"));
    assert!(!d.stripped);
    assert_eq!(d.url.as_str(), "https://github.com/x?utm_source=a");
}

#[test]
fn rule_with_strip_removes_params_and_extra() {
    let d = decide(
        &cfg(),
        &browsers(),
        "https://search.brave.com/?q=1&fbclid=2&ref=x",
    )
    .unwrap();
    assert_eq!(d.browser.id, "brave");
    assert!(d.stripped);
    assert_eq!(d.url.as_str(), "https://search.brave.com/?q=1");
}

#[test]
fn default_browser_uses_global_strip() {
    let d = decide(&cfg(), &browsers(), "https://learn.microsoft.com/?utm_x=1").unwrap();
    assert_eq!(d.browser.id, "edge");
    assert_eq!(d.rule_id, None);
    assert_eq!(d.url.as_str(), "https://learn.microsoft.com/");
}

#[test]
fn fallback_browser_is_used_and_strips() {
    let d = decide(&Loaded::Fallback(b("brave")), &[], "https://x.com/?gclid=1").unwrap();
    assert_eq!(d.browser.id, "brave");
    assert_eq!(d.url.as_str(), "https://x.com/");
}

#[test]
fn unknown_browser_id_errors() {
    assert!(matches!(
        decide(&cfg(), &[b("edge")], "https://github.com/"),
        Err(LinkError::BrowserNotFound(_))
    ));
}
