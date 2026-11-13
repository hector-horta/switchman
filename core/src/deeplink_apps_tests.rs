use crate::deeplinks::CATALOG;
use url::Url;

fn tr(id: &str, link: &str) -> Option<String> {
    let app = CATALOG.iter().find(|a| a.id == id).unwrap();
    app.translate(&Url::parse(link).unwrap())
}

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

#[test]
fn teams_links() {
    assert_eq!(
        tr(
            "teams",
            "https://teams.microsoft.com/l/meetup-join/19%3ameeting_x/0?context=%7b%7d"
        ),
        s("msteams:/l/meetup-join/19%3ameeting_x/0?context=%7b%7d")
    );
    assert_eq!(tr("teams", "https://teams.microsoft.com/v2/"), None);
}

#[test]
fn telegram_links() {
    assert_eq!(
        tr("telegram", "https://t.me/durov"),
        s("tg://resolve?domain=durov")
    );
    assert_eq!(
        tr("telegram", "https://t.me/durov/42"),
        s("tg://resolve?domain=durov&post=42")
    );
    assert_eq!(
        tr("telegram", "https://t.me/+AbCd123"),
        s("tg://join?invite=AbCd123")
    );
    assert_eq!(
        tr("telegram", "https://t.me/joinchat/AbCd123"),
        s("tg://join?invite=AbCd123")
    );
    assert_eq!(tr("telegram", "https://t.me/share/url?url=x"), None);
}

#[test]
fn zoom_links() {
    assert_eq!(
        tr("zoom", "https://us02web.zoom.us/j/81234567890?pwd=abc1"),
        s("zoommtg://zoom.us/join?action=join&confno=81234567890&pwd=abc1")
    );
    assert_eq!(
        tr("zoom", "https://zoom.us/j/81234567890"),
        s("zoommtg://zoom.us/join?action=join&confno=81234567890")
    );
    assert_eq!(tr("zoom", "https://zoom.us/pricing"), None);
}

#[test]
fn slack_links() {
    assert_eq!(
        tr("slack", "https://app.slack.com/client/T0123/C0456"),
        s("slack://channel?team=T0123&id=C0456")
    );
    assert_eq!(
        tr("slack", "https://app.slack.com/client/T0123"),
        s("slack://open?team=T0123")
    );
    assert_eq!(tr("slack", "https://acme.slack.com/archives/C0456"), None);
}

#[test]
fn spotify_links() {
    assert_eq!(
        tr(
            "spotify",
            "https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC?si=abc"
        ),
        s("spotify:track:4uLU6hMCjMI75M1A2tKUQC")
    );
    assert_eq!(
        tr(
            "spotify",
            "https://open.spotify.com/intl-es/album/1ATL5GLyefJaxhQzSPVrLX"
        ),
        s("spotify:album:1ATL5GLyefJaxhQzSPVrLX")
    );
    assert_eq!(tr("spotify", "https://open.spotify.com/search/x"), None);
}

#[test]
fn notion_links() {
    assert_eq!(
        tr("notion", "https://www.notion.so/acme/Doc-123abc?pvs=4"),
        s("notion://www.notion.so/acme/Doc-123abc?pvs=4")
    );
    assert_eq!(
        tr("notion", "https://notion.so/"),
        s("notion://www.notion.so/")
    );
}
