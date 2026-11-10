//! CLI de ferro-link: lo invoca Windows al abrir un enlace y la UI para configurar.
#![cfg_attr(not(test), windows_subsystem = "windows")]
mod commands;
mod config_cmds;

use ferro_link_core::error::LinkError;
use std::process::ExitCode;
use tracing::error;

const USAGE: &str = "uso: ferro-link <open-url URL | test-url URL | detect | set-fallback ID | get-config | set-config --revision R | status | register | unregister>";

/// Despacha el subcomando y devuelve 0 (ok), 1 (error) o 2 (uso incorrecto).
fn main() -> ExitCode {
    init_logging();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("");
    if cmd != "open-url" {
        attach_console();
    }
    let rest: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    let result = match (cmd, rest.as_slice()) {
        ("open-url", [url]) => commands::open_url(url),
        ("test-url", [url]) => commands::test_url(url),
        ("set-fallback", [id]) => commands::set_fallback(id),
        ("set-config", ["--revision", rev]) => config_cmds::set_config(rev),
        ("get-config", []) => config_cmds::get_config(),
        ("status", []) => config_cmds::status(),
        ("detect", []) => commands::detect(),
        ("register", []) => commands::register(),
        ("unregister", []) => commands::unregister(),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!(error = %e, command = cmd, "fallo");
            eprintln!("error: {e}");
            ExitCode::from(match e {
                LinkError::Corrupt(_) => 3,
                LinkError::Conflict => 4,
                _ => 1,
            })
        }
    }
}

/// Registra en `%LOCALAPPDATA%\FerroLink\ferro-link.log` (sin consola visible).
/// Si falla, no hay logger donde informarlo: se sigue sin log.
#[allow(clippy::let_underscore_must_use)]
fn init_logging() {
    let Ok(path) = ferro_link_core::paths::log_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    let _ = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(file))
        .try_init();
}

/// Adjunta la consola del proceso padre para que los mensajes de `println!` sean visibles en el terminal.
fn attach_console() {
    #[cfg(windows)]
    unsafe {
        // SAFETY: AttachConsole no requiere condiciones previas y su fallo es ignorado (no afecta al programa).
        windows_sys::Win32::System::Console::AttachConsole(
            windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS,
        );
    }
}
