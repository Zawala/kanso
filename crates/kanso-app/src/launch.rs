//! Launch at login, with a per-user LaunchAgent.

use std::path::{Path, PathBuf};

use anyhow::Context as _;

pub const LABEL: &str = "app.kanso.Kanso";

/// Escapes text for an XML element.
pub fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// The LaunchAgent property list that runs `program` with `args` at login.
pub fn plist(program_args: &[String]) -> String {
    let args: String = program_args
        .iter()
        .map(|arg| format!("        <string>{}</string>\n", xml_escape(arg)))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
{args}    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>
"#
    )
}

/// How to start this copy of Kanso on `data_dir`: through `open` when it's
/// in an app bundle, otherwise the executable itself.
pub fn program_args(exe: &Path, data_dir: &Path) -> Vec<String> {
    let bundle = exe
        .ancestors()
        .find(|p| p.extension().is_some_and(|ext| ext == "app"));
    let mut args = match bundle {
        Some(app) => vec![
            "/usr/bin/open".into(),
            "-a".into(),
            app.to_string_lossy().into_owned(),
            "--args".into(),
        ],
        None => vec![exe.to_string_lossy().into_owned()],
    };
    args.push("--data-dir".into());
    args.push(data_dir.to_string_lossy().into_owned());
    args
}

/// Installs or removes the LaunchAgent at `path`, which opens Kanso on
/// `data_dir`.
pub fn set_enabled(path: &PathBuf, enabled: bool, data_dir: &Path) -> anyhow::Result<()> {
    if enabled {
        let exe = std::env::current_exe().context("couldn't find the Kanso executable")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, plist(&program_args(&exe, data_dir)))
            .with_context(|| format!("writing {}", path.display()))?;
    } else {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err).with_context(|| format!("removing {}", path.display())),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{plist, program_args, xml_escape};
    use proptest::prelude::*;
    use std::path::Path;

    fn xml_unescape(text: &str) -> String {
        text.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&amp;", "&")
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round-trip: escaping then unescaping gives the text back.
        #[test]
        fn prop_xml_escape_round_trips(text in "\\PC{0,60}") {
            prop_assert_eq!(xml_unescape(&xml_escape(&text)), text);
        }

        /// Invariant: escaped text has no markup characters.
        #[test]
        fn prop_xml_escape_removes_markup(text in "\\PC{0,60}") {
            let escaped = xml_escape(&text);
            prop_assert!(!escaped.contains(['<', '>', '"', '\'']));
            prop_assert!(escaped.match_indices('&').all(|(at, _)| ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"].iter().any(|e| escaped[at..].starts_with(e))));
        }
    }

    #[test]
    fn bundle_uses_open() {
        let data = Path::new("/Users/me/Notes");
        let args = program_args(
            Path::new("/Applications/Kanso.app/Contents/MacOS/kanso"),
            data,
        );
        assert_eq!(
            args,
            [
                "/usr/bin/open",
                "-a",
                "/Applications/Kanso.app",
                "--args",
                "--data-dir",
                "/Users/me/Notes"
            ]
        );
        let args = program_args(Path::new("/Users/me/kanso/target/release/kanso"), data);
        assert_eq!(
            args,
            [
                "/Users/me/kanso/target/release/kanso",
                "--data-dir",
                "/Users/me/Notes"
            ]
        );
        assert!(plist(&args).contains("<string>/Users/me/kanso/target/release/kanso</string>"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round-trip: the arguments the agent passes, read the way Kanso
        /// reads its command line, open the same data folder.
        #[test]
        fn prop_launch_opens_the_same_data_folder(
            dir in "/[^\\x00]{0,40}",
            bundled in any::<bool>(),
        ) {
            let exe = if bundled {
                Path::new("/Applications/Kanso.app/Contents/MacOS/kanso")
            } else {
                Path::new("/usr/local/bin/kanso")
            };
            let args = program_args(exe, Path::new(&dir));
            // `open --args` hands the app whatever follows it.
            let passed: Vec<String> = match args.iter().position(|a| a == "--args") {
                Some(at) => args[at + 1..].to_vec(),
                None => args[1..].to_vec(),
            };
            let parsed = crate::parse_args_from(passed).unwrap();
            prop_assert_eq!(parsed.command, None);
            prop_assert_eq!(parsed.data_dir, Path::new(&dir).to_path_buf());
        }
    }

    /// Oracle: macOS's own plist checker accepts the file.
    #[test]
    fn plutil_accepts_the_plist() {
        let plutil = Path::new("/usr/bin/plutil");
        if !plutil.exists() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent.plist");
        std::fs::write(
            &path,
            plist(&["/Apps/K & K <test>.app".to_owned(), "--flag".to_owned()]),
        )
        .unwrap();
        let status = std::process::Command::new(plutil)
            .arg("-lint")
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            status.status.success(),
            "{}",
            String::from_utf8_lossy(&status.stdout)
        );
    }
}
