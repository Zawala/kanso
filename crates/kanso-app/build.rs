//! On Windows, puts the app icon (assets/icon/kanso.ico) into kanso.exe so
//! Explorer, the taskbar and the window show it, and keeps DirectML from
//! loading with the app. Nothing on other platforms.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // ONNX Runtime, linked in for voice input (kanso-voice), carries
        // DirectML's setup code, which makes the exe import DirectML and
        // Direct3D 12. Voice input runs on the CPU and never calls them, so
        // they load only if called: never, rather than at every launch.
        for dll in ["DirectML.dll", "d3d12.dll"] {
            println!("cargo:rustc-link-arg-bins=/DELAYLOAD:{dll}");
        }
        println!("cargo:rustc-link-arg-bins=delayimp.lib");
    }
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
