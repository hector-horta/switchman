use super::*;

fn m(pattern: &str, url: &str) -> bool {
    HostPattern::parse(pattern)
        .unwrap()
        .matches(&Url::parse(url).unwrap())
}

#[test]
fn matches_exact_host() {
    assert!(m("github.com", "https://github.com/rust-lang"));
}

#[test]
fn matches_subdomains() {
    assert!(m("github.com", "https://gist.github.com/x"));
    assert!(m("*.github.com", "https://api.github.com/"));
}

#[test]
fn rejects_lookalike_hosts() {
    assert!(!m("github.com", "https://notgithub.com/"));
    assert!(!m("github.com", "https://github.com.evil.io/"));
}

#[test]
fn ignores_domain_in_query_or_path() {
    assert!(!m("github.com", "https://evil.com/?r=github.com"));
    assert!(!m("github.com", "https://evil.com/github.com/x"));
}

#[test]
fn case_insensitive_host() {
    assert!(m("GitHub.COM", "https://GITHUB.com/"));
}

#[test]
fn path_prefix() {
    assert!(m(
        "github.com/rust-lang",
        "https://github.com/rust-lang/rust"
    ));
    assert!(!m("github.com/rust-lang", "https://github.com/microsoft/x"));
    assert!(m(
        "github.com/rust-lang/*",
        "https://github.com/rust-lang/cargo"
    ));
}

#[test]
fn accepts_scheme_in_pattern() {
    assert!(m("https://github.com", "https://github.com/"));
}

#[test]
fn idn_pattern_matches_punycode_url() {
    assert!(m("bücher.de", "https://xn--bcher-kva.de/"));
}

#[test]
fn invalid_patterns_are_rejected() {
    assert!(HostPattern::parse("").is_err());
    assert!(HostPattern::parse("   ").is_err());
    assert!(HostPattern::parse("*.").is_err());
    assert!(HostPattern::parse("exa mple.com").is_err());
    assert!(HostPattern::parse("foo*.com").is_err());
}
