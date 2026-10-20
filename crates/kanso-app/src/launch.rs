//! Launch at login. On macOS 13 and later, Kanso installed as an app on
//! its usual data folder is a login item (SMAppService), which System
//! Settings › General › Login Items lists and can turn off; otherwise a
//! per-user LaunchAgent opens it on its data folder. On Windows it is a
//! value under the Run key of HKEY_CURRENT_USER, and on Linux an XDG
//! autostart entry.

use std::path::{Path, PathBuf};

use anyhow::Context as _;

pub const LABEL: &str = label_for(&crate::flavour::CURRENT);

/// The login item's label for `flavour`: its bundle ID.
pub const fn label_for(flavour: &crate::flavour::Flavour) -> &'static str {
    flavour.bundle_id
}

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

/// Whether Kanso can open itself at login here.
pub const SUPPORTED: bool = true;

/// Quotes one argument for a Windows command line as programs built with
/// the C runtime (and Rust's `std::env::args`) read it back: in quotes,
/// with backslashes doubled only before a quote or the closing quote.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn windows_quote(arg: &str) -> String {
    let mut quoted = String::with_capacity(arg.len() + 2);
    quoted.push('"');
    let mut backslashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
                continue;
            }
            _ => {}
        }
        if c != '\\' {
            quoted.extend(std::iter::repeat_n('\\', backslashes));
            backslashes = 0;
            quoted.push(c);
        }
    }
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

/// The command the Run key holds: every argument quoted.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn windows_command(program_args: &[String]) -> String {
    program_args
        .iter()
        .map(|arg| windows_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Quotes an argument for a desktop entry's `Exec` key, then escapes it as
/// a desktop entry string value (the Desktop Entry Specification's two
/// layers of escaping).
pub fn exec_quote(arg: &str) -> String {
    let mut quoted = String::with_capacity(arg.len() + 2);
    quoted.push('"');
    for c in arg.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                quoted.push('\\');
                quoted.push(c);
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted.replace('\\', "\\\\").replace('%', "%%")
}

/// The XDG autostart entry that runs `program_args` at login.
pub fn autostart_entry(program_args: &[String]) -> String {
    let exec: Vec<String> = program_args.iter().map(|arg| exec_quote(arg)).collect();
    format!(
        "[Desktop Entry]\nType=Application\nName=Kanso\nExec={}\nIcon=kanso\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        exec.join(" ")
    )
}

/// What `set_enabled` writes: the LaunchAgent on macOS, the autostart entry
/// elsewhere. In an AppImage, the image rather than its mounted copy.
#[cfg_attr(target_os = "windows", allow(dead_code))]
fn login_item(data_dir: &Path) -> anyhow::Result<String> {
    if !SUPPORTED {
        anyhow::bail!("opening at login isn't available on this system yet");
    }
    let exe = match std::env::var_os("APPIMAGE").filter(|_| cfg!(target_os = "linux")) {
        Some(image) => PathBuf::from(image),
        None => std::env::current_exe().context("couldn't find the Kanso executable")?,
    };
    let args = program_args(&exe, data_dir);
    Ok(if cfg!(target_os = "macos") {
        plist(&args)
    } else {
        autostart_entry(&args)
    })
}

/// Installs or removes what opens Kanso on `data_dir` at login: a login
/// item, a Run key value, or the file at `path` (see `login_item`).
pub fn set_enabled(path: &PathBuf, enabled: bool, data_dir: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    if app_service::usable(data_dir) {
        app_service::set(enabled)?;
        // A LaunchAgent from before would open a second copy.
        return set_file(path, false, data_dir);
    }
    #[cfg(target_os = "windows")]
    {
        let _ = path;
        run_key::set(enabled, data_dir)
    }
    #[cfg(not(target_os = "windows"))]
    set_file(path, enabled, data_dir)
}

#[cfg_attr(target_os = "windows", allow(dead_code))]
fn set_file(path: &PathBuf, enabled: bool, data_dir: &Path) -> anyhow::Result<()> {
    if enabled {
        let item = login_item(data_dir)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, item).with_context(|| format!("writing {}", path.display()))?;
    } else {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err).with_context(|| format!("removing {}", path.display())),
        }
    }
    Ok(())
}

/// The main app as a login item, through SMAppService (macOS 13 and later).
#[cfg(target_os = "macos")]
mod app_service {
    use std::path::Path;

    use objc2::runtime::AnyClass;
    use objc2_service_management::SMAppService;

    /// A login item opens the app with no arguments, so only for the app
    /// bundle on its usual data folder, and only where SMAppService exists.
    pub fn usable(data_dir: &Path) -> bool {
        let bundled = std::env::current_exe().ok().is_some_and(|exe| {
            exe.ancestors()
                .any(|p| p.extension().is_some_and(|e| e == "app"))
        });
        bundled
            && data_dir == crate::paths::default_data_dir()
            && AnyClass::get(c"SMAppService").is_some()
    }

    pub fn set(enabled: bool) -> anyhow::Result<()> {
        // SAFETY: SMAppService exists (`usable`); these calls take no
        // arguments and report failure through NSError.
        let service = unsafe { SMAppService::mainAppService() };
        let result = if enabled {
            unsafe { service.registerAndReturnError() }
        } else {
            unsafe { service.unregisterAndReturnError() }
        };
        match result {
            Ok(()) => Ok(()),
            // Turning off what was never on is done.
            Err(_) if !enabled => Ok(()),
            Err(err) => anyhow::bail!("{}", err.localizedDescription()),
        }
    }
}

/// The Run key value under HKEY_CURRENT_USER.
#[cfg(target_os = "windows")]
mod run_key {
    use std::path::Path;

    use anyhow::Context as _;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
        RegOpenKeyExW, RegSetValueExW,
    };

    const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain([0]).collect()
    }

    pub fn set(enabled: bool, data_dir: &Path) -> anyhow::Result<()> {
        let exe = std::env::current_exe().context("couldn't find the Kanso executable")?;
        let command = super::windows_command(&super::program_args(&exe, data_dir));
        let name = wide(crate::flavour::CURRENT.name);
        let mut key: HKEY = std::ptr::null_mut();
        // SAFETY: the strings are NUL-terminated UTF-16 that outlive the
        // calls, and the key is closed once.
        unsafe {
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                wide(RUN).as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut key,
            );
            if status != ERROR_SUCCESS {
                anyhow::bail!("couldn't open the Run key (error {status})");
            }
            let status = if enabled {
                let data = wide(&command);
                RegSetValueExW(
                    key,
                    name.as_ptr(),
                    0,
                    REG_SZ,
                    data.as_ptr().cast(),
                    (data.len() * 2) as u32,
                )
            } else {
                match RegDeleteValueW(key, name.as_ptr()) {
                    ERROR_FILE_NOT_FOUND => ERROR_SUCCESS,
                    status => status,
                }
            };
            RegCloseKey(key);
            if status != ERROR_SUCCESS {
                anyhow::bail!("couldn't change the Run key (error {status})");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        autostart_entry, exec_quote, plist, program_args, windows_command, windows_quote,
        xml_escape,
    };
    use proptest::prelude::*;
    use std::path::Path;

    /// Reads one argument back the way a desktop environment does: undo the
    /// string escaping, then the `Exec` quoting.
    fn exec_unquote(text: &str) -> Option<String> {
        let mut value = String::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                value.push(chars.next()?);
            } else {
                value.push(c);
            }
        }
        let value = value.replace("%%", "%");
        let inner = value.strip_prefix('"')?.strip_suffix('"')?;
        let mut arg = String::new();
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            match c {
                '\\' => arg.push(chars.next()?),
                '"' | '`' | '$' => return None,
                c => arg.push(c),
            }
        }
        Some(arg)
    }

    /// Reads a Windows command line back the way the C runtime splits it
    /// (the rules `windows_quote` writes for).
    fn windows_split(line: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut chars = line.chars().peekable();
        loop {
            while chars.peek() == Some(&' ') {
                chars.next();
            }
            if chars.peek().is_none() {
                return args;
            }
            let mut arg = String::new();
            let mut quoted = false;
            while let Some(&c) = chars.peek() {
                match c {
                    '\\' => {
                        let mut n = 0;
                        while chars.peek() == Some(&'\\') {
                            chars.next();
                            n += 1;
                        }
                        if chars.peek() == Some(&'"') {
                            arg.extend(std::iter::repeat_n('\\', n / 2));
                            if n % 2 == 1 {
                                arg.push('"');
                                chars.next();
                            }
                        } else {
                            arg.extend(std::iter::repeat_n('\\', n));
                        }
                    }
                    '"' => {
                        quoted = !quoted;
                        chars.next();
                    }
                    ' ' if !quoted => break,
                    c => {
                        arg.push(c);
                        chars.next();
                    }
                }
            }
            args.push(arg);
        }
    }

    #[test]
    fn windows_command_examples() {
        assert_eq!(
            windows_quote(r"C:\Program Files\Kanso\kanso.exe"),
            r#""C:\Program Files\Kanso\kanso.exe""#
        );
        assert_eq!(windows_quote(r"C:\Notes\"), r#""C:\Notes\\""#);
        assert_eq!(windows_quote(r#"say "hi""#), r#""say \"hi\"""#);
        let args = vec![
            r"C:\a b\kanso.exe".to_owned(),
            "--data-dir".to_owned(),
            r"D:\x\".to_owned(),
        ];
        assert_eq!(windows_split(&windows_command(&args)), args);
    }

    proptest! {
        /// Round-trip: Windows splits the command back into the arguments.
        #[test]
        fn prop_windows_command_round_trips(args in proptest::collection::vec("[a-zA-Z0-9 :\\\\\"._-]{0,20}", 1..4)) {
            prop_assert_eq!(windows_split(&windows_command(&args)), args);
        }
    }

    #[test]
    fn autostart_entry_examples() {
        assert_eq!(exec_quote("/opt/kanso/kanso"), "\"/opt/kanso/kanso\"");
        assert_eq!(exec_quote("a b"), "\"a b\"");
        assert_eq!(exec_quote("100%"), "\"100%%\"");
        assert_eq!(exec_quote("$HOME"), "\"\\\\$HOME\"");
        let entry = autostart_entry(&["/opt/kanso".to_owned(), "--data-dir".to_owned()]);
        assert!(entry.starts_with("[Desktop Entry]\n"));
        assert!(entry.contains("\nExec=\"/opt/kanso\" \"--data-dir\"\n"));
    }

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

        /// Round-trip: a desktop environment reads each quoted argument back
        /// as it was, whatever quotes, dollars, backslashes or percents it has.
        #[test]
        fn prop_exec_quote_round_trips(arg in "[^\\x00\\n\\r]{0,60}") {
            prop_assert_eq!(exec_unquote(&exec_quote(&arg)), Some(arg));
        }

        /// Invariant: the entry is one line per key, whatever the arguments.
        #[test]
        fn prop_autostart_entry_keeps_one_exec_line(
            args in proptest::collection::vec("[^\\x00\\n\\r]{0,30}", 1..4),
        ) {
            let entry = autostart_entry(&args);
            prop_assert_eq!(entry.lines().filter(|l| l.starts_with("Exec=")).count(), 1);
            prop_assert_eq!(entry.lines().count(), 7);
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
