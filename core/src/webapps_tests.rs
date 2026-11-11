use super::*;

#[test]
fn split_args_handles_quotes() {
    assert_eq!(
        split_args(" --profile-directory=Default --app-id=abc "),
        ["--profile-directory=Default", "--app-id=abc"]
    );
    assert_eq!(
        split_args(r#"--profile-directory="Profile 1" --app-id=abc"#),
        ["--profile-directory=Profile 1", "--app-id=abc"]
    );
    assert!(split_args("   ").is_empty());
}

#[test]
fn shortcut_with_app_id_becomes_a_destination() {
    let b = from_shortcut(
        "Instagram",
        Some("Helium Apps"),
        r"C:\H\chrome_proxy.exe",
        "--profile-directory=Default --app-id=akpa",
    )
    .unwrap();
    assert_eq!(b.id, "instagram-app");
    assert_eq!(b.name, "Instagram (app de Helium)");
    assert_eq!(b.exe_path, r"C:\H\chrome_proxy.exe");
    assert_eq!(
        b.args,
        [
            "--profile-directory=Default",
            "--app-id=akpa",
            "--app-launch-url-for-shortcuts-menu-item={url}"
        ]
    );
}

#[test]
fn folder_without_apps_suffix_gives_generic_name() {
    let b = from_shortcut(
        "YouTube Music",
        None,
        r"C:\E\msedge_proxy.exe",
        "--app-id=xyz",
    )
    .unwrap();
    assert_eq!(b.name, "YouTube Music (app)");
    assert_eq!(b.id, "youtube-music-app");
    let c = from_shortcut(
        "Notion",
        Some("Programs"),
        r"C:\E\msedge_proxy.exe",
        "--app-id=xyz",
    )
    .unwrap();
    assert_eq!(c.name, "Notion (app)");
}

#[test]
fn shortcut_without_app_id_is_ignored() {
    assert!(from_shortcut("Zen", None, r"C:\Z\zen.exe", "--new-window").is_none());
    assert!(from_shortcut("Rota", None, r"C:\H\chrome_proxy.exe", "--app-id=").is_none());
    assert!(from_shortcut("Sin exe", None, "  ", "--app-id=abc").is_none());
}

#[test]
fn long_names_still_make_valid_ids() {
    let b = from_shortcut(
        &"x".repeat(100),
        None,
        r"C:\H\chrome_proxy.exe",
        "--app-id=abc",
    )
    .unwrap();
    assert!(b.id.len() <= 64);
    assert!(b.id.ends_with("-app"));
}
