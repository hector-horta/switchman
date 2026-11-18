use super::*;

fn redirect(id: &str, from: &str, to: &str) -> Redirect {
    Redirect {
        id: id.into(),
        from: from.into(),
        to: to.into(),
    }
}

fn config(redirects: Vec<Redirect>) -> Config {
    Config {
        default_browser_id: "edge".into(),
        redirects,
        ..Config::default()
    }
}

fn applied(config: &Config, link: &str) -> Option<Target> {
    apply(config, &Url::parse(link).unwrap())
}

#[test]
fn default_reddit_download_page_goes_to_the_home_page() {
    let c = Config::default();
    let home = Some(Target::Link(Url::parse("https://www.reddit.com/").unwrap()));
    assert_eq!(applied(&c, "https://www.reddit.com/mobile/download"), home);
    assert_eq!(applied(&c, "http://reddit.com/mobile/download/"), home);
    assert_eq!(applied(&c, "https://www.reddit.com/r/rust/"), None);
}

#[test]
fn redirect_to_an_app_uri() {
    let c = config(vec![redirect("wa", "www.whatsapp.com", "whatsapp://")]);
    assert_eq!(
        applied(&c, "http://www.whatsapp.com/"),
        Some(Target::App("whatsapp://".into()))
    );
}

#[test]
fn first_matching_redirect_wins_and_bad_ones_are_skipped() {
    let c = config(vec![
        redirect("bad", "a.com", "javascript:alert(1)"),
        redirect("one", "a.com/x", "https://one.com/"),
        redirect("two", "a.com", "https://two.com/"),
    ]);
    let one = Some(Target::Link(Url::parse("https://one.com/").unwrap()));
    let two = Some(Target::Link(Url::parse("https://two.com/").unwrap()));
    assert_eq!(applied(&c, "https://a.com/x/y"), one);
    assert_eq!(applied(&c, "https://a.com/z"), two);
}

#[test]
fn validate_rejects_bad_redirects() {
    assert!(validate(&config(vec![redirect("ok", "a.com/x", "https://b.com/")])).is_ok());
    assert!(validate(&config(vec![redirect("ok", "a.com", "whatsapp://")])).is_ok());
    assert!(validate(&config(vec![redirect("", "a.com", "https://b.com/")])).is_err());
    assert!(validate(&config(vec![redirect("r", "", "https://b.com/")])).is_err());
    assert!(validate(&config(vec![redirect("r", "a.com", "")])).is_err());
    assert!(validate(&config(vec![redirect("r", "a.com", "file:///C:/x")])).is_err());
    assert!(validate(&config(vec![redirect("r", "a.com", "javascript:x")])).is_err());
}

#[test]
fn old_config_gets_the_defaults_but_an_empty_list_stays_empty() {
    let old: Config = serde_json::from_str(r#"{"default_browser_id":"edge"}"#).unwrap();
    assert_eq!(old.redirects, crate::config_types::default_redirects());
    let none: Config =
        serde_json::from_str(r#"{"default_browser_id":"edge","redirects":[]}"#).unwrap();
    assert!(none.redirects.is_empty());
}

#[test]
fn trailing_slash_means_only_the_home_page() {
    let c = config(vec![redirect("wa", "www.whatsapp.com/", "whatsapp://")]);
    let app = Some(Target::App("whatsapp://".into()));
    assert_eq!(applied(&c, "http://www.whatsapp.com/"), app);
    assert_eq!(applied(&c, "https://www.whatsapp.com"), app);
    assert_eq!(applied(&c, "https://www.whatsapp.com/download"), None);
    let site = config(vec![redirect("wa", "www.whatsapp.com", "whatsapp://")]);
    assert_eq!(
        applied(&site, "https://www.whatsapp.com/download"),
        Some(Target::App("whatsapp://".into()))
    );
}
