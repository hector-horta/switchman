//! Incrusta el ícono y los metadatos en `switchman-cli.exe`: es el ejecutable registrado
//! como navegador, así que es el ícono que Windows muestra en Configuración.

fn main() {
    println!("cargo:rerun-if-changed=../assets/switchman.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../assets/switchman.ico")
        .set("ProductName", "Switchman")
        .set("FileDescription", "Switchman: enruta enlaces y quita parámetros de rastreo");
    if let Err(e) = res.compile() {
        println!("cargo:warning=no se pudo incrustar el ícono: {e}");
    }
}
