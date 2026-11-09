use super::*;

fn clean(input: &str) -> String {
    let mut u = Url::parse(input).unwrap();
    strip_tracking(&mut u, &[]);
    u.to_string()
}

#[test]
fn detects_known_params() {
    assert!(is_tracking_param("fbclid", &[]));
    assert!(is_tracking_param("UTM_Source", &[]));
    assert!(is_tracking_param("gclid", &[]));
    assert!(!is_tracking_param("q", &[]));
    assert!(!is_tracking_param("utm", &[]));
}

#[test]
fn extra_params_are_respected() {
    let extra = vec!["ref".to_string()];
    assert!(is_tracking_param("REF", &extra));
    assert!(!is_tracking_param("ref", &[]));
}

#[test]
fn strips_utm_and_keeps_others() {
    assert_eq!(
        clean("https://github.com/rust-lang/rust?utm_source=x&tab=readme&fbclid=1"),
        "https://github.com/rust-lang/rust?tab=readme"
    );
}

#[test]
fn removes_query_when_all_tracking() {
    assert_eq!(
        clean("https://example.com/a?utm_medium=y&gclid=2"),
        "https://example.com/a"
    );
}

#[test]
fn keeps_fragment() {
    assert_eq!(
        clean("https://example.com/a?utm_x=1&id=7#top"),
        "https://example.com/a?id=7#top"
    );
}

#[test]
fn keeps_percent_encoding() {
    assert_eq!(
        clean("https://example.com/s?q=a%26b+c&utm_x=1"),
        "https://example.com/s?q=a%26b+c"
    );
}

#[test]
fn returns_false_when_unchanged() {
    let mut u = Url::parse("https://example.com/?q=1").unwrap();
    assert!(!strip_tracking(&mut u, &[]));
    assert_eq!(u.as_str(), "https://example.com/?q=1");
}

#[test]
fn returns_true_when_changed() {
    let mut u = Url::parse("https://example.com/?fbclid=1").unwrap();
    assert!(strip_tracking(&mut u, &[]));
    assert_eq!(u.as_str(), "https://example.com/");
}

#[test]
fn no_query_is_noop() {
    assert_eq!(
        clean("https://example.com/path"),
        "https://example.com/path"
    );
}
