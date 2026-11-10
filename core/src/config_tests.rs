// La limpieza de directorios temporales es best-effort.
#![allow(clippy::let_underscore_must_use)]
use super::*;
use crate::config_types::Rule;
use std::fs;

fn browser(id: &str) -> Browser {
    Browser {
        id: id.into(),
        name: id.into(),
        exe_path: format!("C:/b/{id}.exe"),
        args: vec![],
    }
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
        custom_browsers: vec![browser("zen")],
        extra_tracking_params: vec![],
    }
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ferro-link-cfg-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn valid_config_passes() {
    assert!(validate(&sample()).is_ok());
}

#[test]
fn empty_default_browser_fails() {
    let mut c = sample();
    c.default_browser_id = "".into();
    assert!(validate(&c).is_err());
}

#[test]
fn invalid_rule_pattern_fails() {
    let mut c = sample();
    c.rules[0].pattern = "foo*.com".into();
    assert!(validate(&c).is_err());
}

#[test]
fn invalid_id_chars_fail() {
    let mut c = sample();
    c.rules[0].browser_id = "chrome; rm".into();
    assert!(validate(&c).is_err());

    let mut c2 = sample();
    c2.custom_browsers[0].id = "x".repeat(65);
    assert!(validate(&c2).is_err());
}

#[test]
fn empty_exe_path_fails() {
    let mut c = sample();
    c.custom_browsers[0].exe_path = "  ".into();
    assert!(validate(&c).is_err());
}

#[test]
fn save_then_load_roundtrip() {
    let dir = temp_dir("rt");
    let p = dir.join("config.json");
    save(&p, &sample()).unwrap();
    assert_eq!(load(&p).unwrap(), sample());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_optional_fields_use_defaults() {
    let dir = temp_dir("defaults");
    let p = dir.join("config.json");
    fs::write(&p, r#"{"default_browser_id":"edge"}"#).unwrap();
    let c = load(&p).unwrap();
    assert!(c.strip_tracking);
    assert!(c.rules.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn effective_uses_config_when_valid() {
    let dir = temp_dir("eff1");
    let cp = dir.join("config.json");
    let fp = dir.join("fallback.json");
    save(&cp, &sample()).unwrap();
    assert_eq!(load_effective(&cp, &fp).unwrap(), Loaded::Config(sample()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn effective_uses_fallback_when_config_corrupt() {
    let dir = temp_dir("eff2");
    let cp = dir.join("config.json");
    let fp = dir.join("fallback.json");
    fs::write(&cp, "{ broken").unwrap();
    save_fallback(&fp, &browser("brave")).unwrap();
    assert_eq!(
        load_effective(&cp, &fp).unwrap(),
        Loaded::Fallback(browser("brave"))
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn effective_errors_when_both_missing() {
    let dir = temp_dir("eff3");
    assert!(load_effective(&dir.join("a.json"), &dir.join("b.json")).is_err());
    let _ = fs::remove_dir_all(&dir);
}
