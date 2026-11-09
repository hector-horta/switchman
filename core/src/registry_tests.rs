use super::*;

#[test]
fn exe_from_quoted_command() {
    let cmd = r#""C:\Program Files\Google\Chrome\Application\chrome.exe" --single-argument %1"#;
    assert_eq!(
        exe_from_command(cmd).as_deref(),
        Some(r"C:\Program Files\Google\Chrome\Application\chrome.exe")
    );
}

#[test]
fn exe_from_unquoted_command() {
    let cmd = r"C:\Apps\zen.exe -osint -url %1";
    assert_eq!(exe_from_command(cmd).as_deref(), Some(r"C:\Apps\zen.exe"));
    assert_eq!(exe_from_command("   "), None);
    assert_eq!(exe_from_command(r#""unterminated"#), None);
}

#[test]
fn slug_is_a_valid_id() {
    assert_eq!(slug("Google Chrome"), "google-chrome");
    assert_eq!(
        slug("  Firefox-308046B0AF4A39CB "),
        "firefox-308046b0af4a39cb"
    );
    assert_eq!(slug("Ünïcode!!Browser"), "n-code-browser");
    assert!(slug(&"x".repeat(100)).len() <= 64);
}
