//! Compiles the built-in color and icon themes in (`themes/`, the
//! kanso-themes submodule's: `themes::BUILTIN` and
//! `icon_themes::BUILTIN_FILES`). On Windows, also puts the app icon
//! (assets/icon/kanso.ico) into kanso.exe so Explorer, the taskbar and the
//! window show it, and keeps DirectML from loading with the app.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    builtin_themes();
    builtin_icon_themes();
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

/// The order the Themes page had before this table was generated: Kanso's
/// own first, then each family's light theme beside its dark one. New themes
/// follow in `index.json`'s order, so adding one only needs a submodule bump.
const KEEP_FIRST: &[&str] = &[
    "kanso-light.json",
    "kanso-dark.json",
    "default-light.json",
    "default-dark.json",
    "github-light.json",
    "github-dark.json",
    "solarized-light.json",
    "solarized-dark.json",
    "gruvbox-light.json",
    "gruvbox-dark.json",
    "catppuccin-latte.json",
    "catppuccin-mocha.json",
    "rose-pine-dawn.json",
    "rose-pine.json",
    "kanagawa-lotus.json",
    "kanagawa-wave.json",
    "kanagawa-dragon.json",
    "nord.json",
    "dracula.json",
    "tokyo-night.json",
];

/// Writes `builtin_themes.rs` into `OUT_DIR`: the color themes in
/// `themes/index.json`, by file name, as `include_str!`s. The first two are
/// Kanso's own light and dark themes, which the defaults fall back to.
fn builtin_themes() {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.join("../../themes");
    // The themes are the kanso-themes repository, checked out as a
    // submodule; without it Kanso would build with no themes at all.
    assert!(
        root.is_dir(),
        "themes is missing: run `git submodule update --init` (the themes are the kanso-themes submodule)"
    );
    let index = root.join("index.json");
    println!("cargo:rerun-if-changed={}", root.display());
    println!("cargo:rerun-if-changed={}", index.display());
    let text = std::fs::read_to_string(&index)
        .unwrap_or_else(|err| panic!("{} couldn't be read: {err}", index.display()));
    let value: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("{} isn't valid JSON: {err}", index.display()));
    let listed: Vec<&str> = value
        .get("themes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{} has no themes list", index.display()))
        .iter()
        .map(|theme| {
            theme
                .get("file")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("{} has a theme with no file", index.display()))
        })
        .collect();

    // The previous order for the themes that had one, then the rest as the
    // index lists them.
    let known: BTreeSet<&str> = listed.iter().copied().collect();
    assert_eq!(
        known.len(),
        listed.len(),
        "{} lists a theme twice",
        index.display()
    );
    let mut ordered: Vec<&str> = KEEP_FIRST
        .iter()
        .copied()
        .filter(|file| known.contains(file))
        .collect();
    ordered.extend(
        listed
            .iter()
            .copied()
            .filter(|file| !KEEP_FIRST.contains(file)),
    );
    assert_eq!(
        ordered.len(),
        listed.len(),
        "{} lists a theme the table doesn't",
        index.display()
    );
    assert_eq!(
        ordered.first().copied(),
        Some("kanso-light.json"),
        "BUILTIN[0] must be Kanso's light theme"
    );
    assert_eq!(
        ordered.get(1).copied(),
        Some("kanso-dark.json"),
        "BUILTIN[1] must be Kanso's dark theme"
    );

    let mut code = String::from(
        "/// The built-in themes' files, in the order the Themes page shows them:\n\
         /// Kanso's own first, then each family's light theme beside its dark one.\n\
         pub const BUILTIN: &[(&str, &str)] = &[\n",
    );
    for file in ordered {
        let path = root.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        writeln!(
            code,
            "    ({file:?}, include_str!({:?})),",
            path.display().to_string()
        )
        .unwrap();
    }
    code.push_str("];\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("builtin_themes.rs");
    std::fs::write(out, code).expect("writing builtin_themes.rs");
}

/// Writes `builtin_icon_themes.rs` into `OUT_DIR`: for each folder of
/// `themes/icon-themes/` holding a theme, its ID, its JSON and every SVG under it,
/// by path, as `include_bytes!`s. Only `.json` and `.svg` files go in.
fn builtin_icon_themes() {
    use std::fmt::Write as _;
    use std::path::{Path, PathBuf};

    fn svgs(dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = read.filter_map(Result::ok).map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                svgs(&path, &format!("{prefix}{name}/"), out);
            } else if name.ends_with(".svg") {
                out.push((format!("{prefix}{name}"), path));
            }
        }
    }

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.join("../../themes/icon-themes");
    // The themes are the kanso-themes repository, checked out as a
    // submodule; without it Kanso would build with no themes at all.
    assert!(
        root.is_dir(),
        "themes/icon-themes is missing: run `git submodule update --init` (the themes are the kanso-themes submodule)"
    );
    println!("cargo:rerun-if-changed={}", root.display());
    let mut code = String::from(
        "/// The built-in icon themes: (ID, theme JSON, SVGs by path).\n\
         pub const BUILTIN_FILES: &[BuiltinTheme] = &[\n",
    );
    let mut folders: Vec<PathBuf> = std::fs::read_dir(&root)
        .map(|read| read.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    folders.sort();
    for folder in folders.into_iter().filter(|p| p.is_dir()) {
        let id = folder.file_name().unwrap().to_string_lossy().into_owned();
        // The schema's folder isn't a theme.
        if id == "schema" {
            continue;
        }
        let json = std::fs::read_dir(&folder)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|x| x == "json"));
        let Some(json) = json else {
            continue;
        };
        println!("cargo:rerun-if-changed={}", folder.display());
        println!("cargo:rerun-if-changed={}", json.display());
        let mut files = Vec::new();
        svgs(&folder, "", &mut files);
        writeln!(
            code,
            "    ({id:?}, include_str!({:?}), &[",
            json.display().to_string()
        )
        .unwrap();
        for (rel, path) in files {
            writeln!(
                code,
                "        ({rel:?}, include_bytes!({:?})),",
                path.display().to_string()
            )
            .unwrap();
        }
        code.push_str("    ]),\n");
    }
    code.push_str("];\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("builtin_icon_themes.rs");
    std::fs::write(out, code).expect("writing builtin_icon_themes.rs");
}
