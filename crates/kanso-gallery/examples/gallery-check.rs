//! Checks a builder-generated `gallery/index.json` with Kanso's own gallery
//! parser and SVG sanitizer. `kanso-themes` runs this from its validate job
//! once its pinned Kanso commit has it:
//!
//! ```text
//! gallery-check --index /tmp/gallery/index.json --root . --out /tmp/gallery
//! ```
//!
//! It needs no signature and no network: the index was just built from the
//! tree it is checked against, and the point is the parser and sanitizer,
//! not the signature (the signing job does that separately).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use semver::Version;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("gallery-check: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut index_path = None;
    let mut root = PathBuf::from(".");
    let mut out = PathBuf::from("gallery");
    let mut at = 0;
    while at + 1 < args.len() {
        match args[at].as_str() {
            "--index" => index_path = Some(PathBuf::from(&args[at + 1])),
            "--root" => root = PathBuf::from(&args[at + 1]),
            "--out" => out = PathBuf::from(&args[at + 1]),
            other => return Err(format!("unknown argument {other}")),
        }
        at += 2;
    }
    let index_path = index_path.ok_or("pass --index PATH")?;
    let bytes =
        std::fs::read(&index_path).map_err(|err| format!("{}: {err}", index_path.display()))?;
    let index: kanso_gallery::Index =
        serde_json::from_slice(&bytes).map_err(|err| format!("the index doesn't parse: {err}"))?;
    index
        .check(&Version::new(0, 0, 0))
        .map_err(|err| format!("the index doesn't validate: {err}"))?;

    let mut files = 0usize;
    let mut previews = 0usize;
    for theme in &index.themes {
        if let Some(file) = &theme.file {
            let path = root.join(&file.path);
            check_pin(file, &path, kanso_gallery::MAX_COLOR_BYTES)?;
            files += 1;
        }
        if let Some(bundle) = &theme.bundle {
            let path = out.join(bundle.path.trim_start_matches("gallery/"));
            check_pin(bundle, &path, kanso_gallery::MAX_BUNDLE_BYTES)?;
            files += 1;
        }
        for preview in &theme.preview {
            let path = root.join(&preview.path);
            check_pin(preview, &path, kanso_gallery::MAX_PREVIEW_BYTES)?;
            let svg = std::fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?;
            kanso_gallery::sanitize_svg(&svg).map_err(|err| format!("{}: {err}", preview.path))?;
            previews += 1;
        }
    }

    let mut svgs = Vec::new();
    walk(&root.join("icon-themes"), &mut svgs, 0)?;
    let mut sanitized = 0usize;
    for path in svgs.iter().filter(|path| {
        path.extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
    }) {
        let bytes = std::fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
        kanso_gallery::sanitize_svg(&bytes).map_err(|err| format!("{}: {err}", path.display()))?;
        sanitized += 1;
    }
    println!(
        "gallery-check: {} themes, {} pinned files, {} previews, {} SVGs sanitize",
        index.themes.len(),
        files,
        previews,
        sanitized
    );
    Ok(())
}

fn check_pin(pin: &kanso_gallery::FileRef, path: &Path, cap: u64) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
    kanso_gallery::check_pin(pin, &bytes, cap).map_err(|err| format!("{}: {err}", pin.path))
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<(), String> {
    if depth > 16 {
        return Err(format!("{} nests too deeply", dir.display()));
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|err| err.to_string())?;
        if meta.is_symlink() {
            return Err(format!("{} is a link", path.display()));
        }
        if meta.is_dir() {
            walk(&path, out, depth + 1)?;
        } else if meta.is_file() {
            out.push(path);
        }
    }
    Ok(())
}
