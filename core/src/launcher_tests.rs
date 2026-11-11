use super::*;

fn browser(exe: &Path) -> Browser {
    Browser {
        id: "x".into(),
        name: "x".into(),
        exe_path: exe.to_string_lossy().into_owned(),
        args: vec![],
    }
}

fn url() -> Url {
    Url::parse("https://example.com/").unwrap()
}

#[test]
fn same_exe_ignores_case_and_separators() {
    assert!(same_exe(Path::new(r"C:\A\B.exe"), Path::new("c:/a/b.EXE")));
    assert!(!same_exe(
        Path::new(r"C:\A\B.exe"),
        Path::new(r"C:\A\C.exe")
    ));
}

#[test]
fn refuses_to_launch_itself() {
    let me = std::env::current_exe().unwrap();
    let r = launch(&browser(&me), &url(), &me);
    assert!(matches!(r, Err(LinkError::SelfLaunch)));
}

#[test]
fn missing_exe_is_browser_not_found() {
    let me = std::env::current_exe().unwrap();
    let r = launch(&browser(Path::new(r"Z:\nope\missing.exe")), &url(), &me);
    assert!(matches!(r, Err(LinkError::BrowserNotFound(_))));
}

#[test]
fn url_is_appended_without_placeholder() {
    let args = vec!["--new-window".to_string()];
    assert_eq!(
        launch_args(&args, "https://a.com/"),
        ["--new-window", "https://a.com/"]
    );
    assert_eq!(launch_args(&[], "https://a.com/"), ["https://a.com/"]);
}

#[test]
fn placeholder_is_replaced_inside_the_argument() {
    let args = vec![
        "--app-id=abc".to_string(),
        "--app-launch-url-for-shortcuts-menu-item={url}".to_string(),
    ];
    assert_eq!(
        launch_args(&args, "https://www.instagram.com/p/x/?a=1&b=2"),
        [
            "--app-id=abc",
            "--app-launch-url-for-shortcuts-menu-item=https://www.instagram.com/p/x/?a=1&b=2"
        ]
    );
}
