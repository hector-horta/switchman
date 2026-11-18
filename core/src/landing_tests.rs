use super::*;

fn rewritten(link: &str) -> Option<String> {
    rewrite(&Url::parse(link).unwrap()).map(|u| u.to_string())
}

#[test]
fn reddit_download_page_becomes_the_home_page() {
    let home = Some("https://www.reddit.com/".to_string());
    assert_eq!(rewritten("https://www.reddit.com/mobile/download"), home);
    assert_eq!(rewritten("http://reddit.com/mobile/download/"), home);
    assert_eq!(
        rewritten("https://www.reddit.com/mobile/download?utm_source=x"),
        home
    );
}

#[test]
fn other_pages_are_left_alone() {
    assert_eq!(rewritten("https://www.reddit.com/r/rust/"), None);
    assert_eq!(rewritten("https://www.reddit.com/mobile/downloads"), None);
    assert_eq!(rewritten("https://notreddit.com/mobile/download"), None);
}
