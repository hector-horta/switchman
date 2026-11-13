use super::*;

#[test]
fn system_schemes_are_hidden() {
    assert!(is_system_scheme("https"));
    assert!(is_system_scheme("ms-settings"));
    assert!(is_system_scheme("microsoft.windows.camera"));
    assert!(is_system_scheme("windowsdefender"));
    assert!(!is_system_scheme("steam"));
    assert!(!is_system_scheme("whatsapp"));
}

#[test]
fn only_plain_scheme_names_are_looked_up() {
    assert!(!scheme_registered(r"whatsapp\shell"));
    assert!(!scheme_registered(""));
    assert!(!scheme_registered(".txt"));
}

#[test]
fn uris_with_browser_or_file_schemes_are_refused() {
    assert!(open_uri("https://example.com").is_err());
    assert!(open_uri("file:///C:/Windows").is_err());
    assert!(open_uri("sin-protocolo").is_err());
}

#[test]
fn listable_hides_noise_and_keeps_apps() {
    for app in [
        "steam",
        "whatsapp-smb",
        "spotify",
        "bitwarden",
        "nordvpn",
        "claude",
        "battlenet",
    ] {
        assert!(is_listable(app), "{app} debería listarse");
    }
    for noise in [
        "explorer.zipselection",
        "com.epicgames.launcher",
        "xbox-profile",
        "winscp-sftp",
        "onenote-cmd",
        "msgamepass",
        "google-chrome",
        "firefoxurl-f0dc299d809b9700",
        "sftp",
        "magnet",
        "web+msteams",
        "discord-548703355543748619",
        "https",
        "ms-settings",
    ] {
        assert!(!is_listable(noise), "{noise} no debería listarse");
    }
}
