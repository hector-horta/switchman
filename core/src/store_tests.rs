use super::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ferro-link-test-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn roundtrip_creates_parent_dirs() {
    let dir = temp_dir("roundtrip");
    let path = dir.join("nested").join("a.json");
    let mut map = BTreeMap::new();
    map.insert("k".to_string(), 7u32);
    write_json_atomic(&path, &map).unwrap();
    let back: BTreeMap<String, u32> = read_json(&path).unwrap();
    assert_eq!(back, map);
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    assert!(!std::path::PathBuf::from(tmp).exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn overwrites_existing_file() {
    let dir = temp_dir("overwrite");
    let path = dir.join("b.json");
    write_json_atomic(&path, &1u32).unwrap();
    write_json_atomic(&path, &2u32).unwrap();
    let back: u32 = read_json(&path).unwrap();
    assert_eq!(back, 2);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn corrupt_json_is_config_error() {
    let dir = temp_dir("corrupt");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("c.json");
    fs::write(&path, "{ not json").unwrap();
    let r: Result<u32> = read_json(&path);
    assert!(matches!(r, Err(LinkError::Config(_))));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_file_is_io_error() {
    let r: Result<u32> = read_json(Path::new("Z:/definitely/missing/file.json"));
    assert!(matches!(r, Err(LinkError::Io(_))));
}
