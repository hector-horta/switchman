// La limpieza de directorios temporales es best-effort.
#![allow(clippy::let_underscore_must_use)]

use super::*;
use crate::config_types::Rule;
use std::fs;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("switchman-rev-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sample() -> Config {
    Config {
        default_browser_id: "edge".into(),
        strip_tracking: true,
        rules: vec![Rule {
            id: "r1".into(),
            pattern: "github.com".into(),
            browser_id: "chrome".into(),
            strip_tracking: true,
        }],
        custom_browsers: vec![],
        extra_tracking_params: vec![],
        app_links: vec![],
        custom_deeplinks: vec![],
    }
}

#[test]
fn missing_file_reads_default() {
    let path = temp_dir("missing").join("config.json");
    let v = read_versioned(&path).unwrap();
    assert_eq!(v.revision, NO_FILE);
    assert_eq!(v.config, Config::default());
}

#[test]
fn corrupt_file_is_corrupt_error() {
    let path = temp_dir("corrupt").join("config.json");
    fs::write(&path, "{ no es json").unwrap();
    assert!(matches!(read_versioned(&path), Err(LinkError::Corrupt(_))));
}

#[test]
fn invalid_config_is_corrupt_error() {
    let path = temp_dir("invalid").join("config.json");
    fs::write(&path, r#"{"default_browser_id":""}"#).unwrap();
    assert!(matches!(read_versioned(&path), Err(LinkError::Corrupt(_))));
}

#[test]
fn save_then_read_roundtrip() {
    let path = temp_dir("roundtrip").join("config.json");
    let rev = save_versioned(&path, &sample(), NO_FILE).unwrap();
    let v = read_versioned(&path).unwrap();
    assert_eq!(v.config, sample());
    assert_eq!(v.revision, rev);
    assert_ne!(rev, NO_FILE);
}

#[test]
fn stale_revision_is_conflict() {
    let path = temp_dir("conflict").join("config.json");
    let rev = save_versioned(&path, &sample(), NO_FILE).unwrap();
    fs::write(&path, r#"{"default_browser_id":"chrome"}"#).unwrap();
    assert!(matches!(
        save_versioned(&path, &sample(), &rev),
        Err(LinkError::Conflict)
    ));
}

#[test]
fn creating_over_existing_file_is_conflict() {
    let path = temp_dir("exists").join("config.json");
    save_versioned(&path, &sample(), NO_FILE).unwrap();
    assert!(matches!(
        save_versioned(&path, &sample(), NO_FILE),
        Err(LinkError::Conflict)
    ));
}

#[test]
fn invalid_config_is_not_saved() {
    let path = temp_dir("nosave").join("config.json");
    let bad = Config::default();
    assert!(matches!(
        save_versioned(&path, &bad, NO_FILE),
        Err(LinkError::Config(_))
    ));
    assert!(!path.exists());
}

#[test]
fn revision_is_stable_and_content_based() {
    assert_eq!(revision_of(b"abc"), revision_of(b"abc"));
    assert_ne!(revision_of(b"abc"), revision_of(b"abd"));
    assert_ne!(revision_of(b""), NO_FILE);
}

#[test]
fn default_config_matches_empty_json() {
    let empty: Config = serde_json::from_str(r#"{"default_browser_id":""}"#).unwrap();
    assert_eq!(Config::default(), empty);
    assert!(Config::default().strip_tracking);
}

#[test]
fn reset_moves_corrupt_file_to_bak() {
    let path = temp_dir("reset").join("config.json");
    fs::write(&path, "{ roto").unwrap();
    assert_eq!(reset(&path).unwrap(), NO_FILE);
    assert!(!path.exists());
    assert_eq!(
        fs::read_to_string(path.with_extension("json.bak")).unwrap(),
        "{ roto"
    );
    save_versioned(&path, &sample(), NO_FILE).unwrap();
}

#[test]
fn reset_without_file_is_ok() {
    let path = temp_dir("reset-missing").join("config.json");
    assert_eq!(reset(&path).unwrap(), NO_FILE);
}
