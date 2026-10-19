//! On Windows, puts the app icon (assets/icon/kanso.ico) into kanso.exe so
//! Explorer, the taskbar and the window show it. Nothing on other platforms.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows_icon();
    }
}

#[cfg(windows)]
fn windows_icon() {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let icon = manifest_dir.join("../../assets/icon/kanso.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    let icon = icon.canonicalize().expect("assets/icon/kanso.ico exists");
    // rc.exe wants the path as a C string, without the `\\?\` prefix.
    let icon = icon.to_string_lossy();
    let icon = icon.trim_start_matches(r"\\?\").replace('\\', "\\\\");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let rc = out_dir.join("kanso.rc");
    std::fs::write(&rc, format!("1 ICON \"{icon}\"\n")).expect("writing kanso.rc");
    embed_resource::compile(&rc, embed_resource::NONE)
        .manifest_optional()
        .expect("compiling the icon resource");
}
