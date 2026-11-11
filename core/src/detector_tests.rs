use super::*;

fn env(name: &str) -> Option<String> {
    match name {
        "ProgramFiles" => Some(r"C:\PF".into()),
        "ProgramFiles(x86)" => Some(r"C:\PF86".into()),
        "LOCALAPPDATA" => Some(r"C:\L".into()),
        _ => None,
    }
}

fn b(id: &str, exe: &str) -> Browser {
    Browser {
        id: id.into(),
        name: id.into(),
        exe_path: exe.into(),
        args: vec![],
    }
}

#[test]
fn expands_known_vars() {
    assert_eq!(
        expand_vars(r"%ProgramFiles%\x.exe", &env),
        Some(r"C:\PF\x.exe".to_string())
    );
    assert_eq!(
        expand_vars(r"%ProgramFiles(x86)%\y.exe", &env),
        Some(r"C:\PF86\y.exe".to_string())
    );
}

#[test]
fn unknown_var_is_none() {
    assert_eq!(expand_vars(r"%NOPE%\x.exe", &env), None);
    assert_eq!(expand_vars(r"%ProgramFiles\x.exe", &env), None);
}

#[test]
fn detects_only_existing_paths() {
    let found = detect_known(&env, &|p: &Path| {
        p.to_string_lossy().ends_with("msedge.exe") || p.to_string_lossy().ends_with("zen.exe")
    });
    let ids: Vec<&str> = found.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, vec!["edge", "zen"]);
    assert_eq!(
        found[0].exe_path,
        r"C:\PF86\Microsoft\Edge\Application\msedge.exe"
    );
}

#[test]
fn detects_nothing_when_nothing_exists() {
    assert!(detect_known(&env, &|_: &Path| false).is_empty());
}

#[test]
fn merge_dedupes_by_id_and_path_keeping_first() {
    let merged = merge(vec![
        vec![b("zen", r"C:\Z\zen.exe")],
        vec![
            b("chrome", r"C:\c.exe"),
            b("ZEN", r"D:\other.exe"),
            b("dup", r"c:\Z\ZEN.exe"),
        ],
    ]);
    let ids: Vec<&str> = merged.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, vec!["zen", "chrome"]);
}

#[test]
fn merge_keeps_webapps_sharing_the_browser_exe() {
    let app = |id: &str, app_id: &str| Browser {
        id: id.into(),
        name: id.into(),
        exe_path: r"C:\Helium\chrome_proxy.exe".into(),
        args: vec![format!("--app-id={app_id}")],
    };
    let merged = merge(vec![
        vec![app("instagram-app", "aaa"), app("x-app", "bbb")],
        vec![app("instagram-app", "aaa")],
    ]);
    let ids: Vec<&str> = merged.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, ["instagram-app", "x-app"]);
}
