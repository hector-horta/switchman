//! Límite de líneas por archivo (CLAUDE.md: servicios <= 300). Los *_tests.rs no cuentan.

use std::fs;
use std::path::{Path, PathBuf};

const MAX_LINES: usize = 300;

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && !path.to_string_lossy().ends_with("_tests.rs")
        {
            out.push(path);
        }
    }
}

#[test]
fn source_files_respect_loc_limit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();

    rust_sources(&root.join("core").join("src"), &mut files);
    rust_sources(&root.join("cli").join("src"), &mut files);

    assert!(!files.is_empty(), "no se encontraron fuentes");

    let too_long: Vec<String> = files
        .iter()
        .filter_map(|f| {
            let lines = fs::read_to_string(f).unwrap().lines().count();
            (lines > MAX_LINES).then(|| format!("{} ({} líneas)", f.display(), lines))
        })
        .collect();

    assert!(
        too_long.is_empty(),
        "archivos sobre {MAX_LINES} líneas: {too_long:?}"
    );
}
