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
