//! Where Kanso is installed, whether it can update itself there, and the
//! swap that puts a staged update in its place.
//!
//! The swap is two renames on one volume: what runs now moves aside, the
//! staged update moves in, and if the second rename fails the first is
//! undone. A running program may be renamed on every system Kanso ships
//! for, so the swap happens while the old Kanso is quitting; the new one
//! then waits for its data-folder lock (`--after-update`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::platform::Platform;
use crate::stage::Pending;

/// The staging folder's name, beside what it replaces.
pub const STAGE_DIR_NAME: &str = ".kanso-update";

/// An install Kanso can update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Install {
    /// The `.app` on macOS, the executable elsewhere.
    pub target: PathBuf,
    /// Beside the target, on its volume.
    pub stage_dir: PathBuf,
    pub platform: Platform,
}

/// What the app knows about itself that decides whether it may update.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocateOptions {
    /// Kanso Dev: never updated from releases.
    pub dev_flavour: bool,
    /// A debug build.
    pub debug_build: bool,
    /// Running as an AppImage, whose image the updater doesn't replace.
    pub appimage: bool,
}

/// Why this copy of Kanso can't update itself; the app offers the release
/// page instead and says why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotInstallable {
    DevFlavour,
    DevBuild,
    CargoBuild,
    UnsupportedSystem,
    NotInBundle,
    Translocated,
    DiskImage,
    AppImage,
    NotWritable(PathBuf),
}

impl NotInstallable {
    /// One sentence for the Updates page.
    pub fn reason(&self) -> String {
        match self {
            NotInstallable::DevFlavour => "Development build — updates are off.".into(),
            NotInstallable::DevBuild | NotInstallable::CargoBuild => {
                "This copy was built from source, so it doesn't replace itself. Download the release instead.".into()
            }
            NotInstallable::UnsupportedSystem => "There are no release builds for this system.".into(),
            NotInstallable::NotInBundle => "Kanso isn't running from Kanso.app, so it can't replace itself.".into(),
            NotInstallable::Translocated => {
                "macOS is running Kanso from a temporary copy. Move Kanso.app to Applications, open it from there, and updates install themselves.".into()
            }
            NotInstallable::DiskImage => "Kanso is running from its disk image. Drag it to Applications first.".into(),
            NotInstallable::AppImage => "AppImages update by downloading the new image.".into(),
            NotInstallable::NotWritable(dir) => format!(
                "Kanso can't write to {}, so it can't replace itself. Download the update instead.",
                dir.display()
            ),
        }
    }
}

/// The install `exe` belongs to, if this copy may update itself there:
/// not Kanso Dev, not a debug or `cargo` build, and in a folder it can
/// write to. On macOS that is the enclosing `.app`.
pub fn locate(
    exe: &Path,
    platform: Option<Platform>,
    options: LocateOptions,
) -> Result<Install, NotInstallable> {
    if options.dev_flavour {
        return Err(NotInstallable::DevFlavour);
    }
    if options.debug_build {
        return Err(NotInstallable::DevBuild);
    }
    let platform = platform.ok_or(NotInstallable::UnsupportedSystem)?;
    if is_cargo_build(exe) {
        return Err(NotInstallable::CargoBuild);
    }
    if options.appimage {
        return Err(NotInstallable::AppImage);
    }
    let target = if platform.is_bundle() {
        let app = exe
            .ancestors()
            .find(|p| p.extension().is_some_and(|e| e == "app"))
            .ok_or(NotInstallable::NotInBundle)?
            .to_path_buf();
        let text = app.to_string_lossy();
        if text.contains("/AppTranslocation/") {
            return Err(NotInstallable::Translocated);
        }
        if text.starts_with("/Volumes/") && !writable_dir(app.parent().unwrap_or(&app)) {
            return Err(NotInstallable::DiskImage);
        }
        app
    } else {
        exe.to_path_buf()
    };
    let parent = target
        .parent()
        .ok_or(NotInstallable::NotInBundle)?
        .to_path_buf();
    if !writable_dir(&parent) {
        return Err(NotInstallable::NotWritable(parent));
    }
    Ok(Install {
        stage_dir: parent.join(STAGE_DIR_NAME),
        target,
        platform,
    })
}

/// Whether `exe` sits in a Cargo target folder: `target/debug`,
/// `target/release` or `target/<triple>/release`.
pub fn is_cargo_build(exe: &Path) -> bool {
    let parts: Vec<String> = exe
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts
        .windows(2)
        .any(|w| w[0] == "target" && (w[1] == "debug" || w[1] == "release"))
        || parts.windows(3).any(|w| {
            w[0] == "target" && (w[2] == "debug" || w[2] == "release") && w[1].contains('-')
        })
}

/// Whether a file can be created in `dir` (tried, then removed).
pub fn writable_dir(dir: &Path) -> bool {
    let probe = dir.join(format!(".kanso-write-test-{}", std::process::id()));
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(_) => {
            fs::remove_file(&probe).ok();
            true
        }
        Err(_) => false,
    }
}

/// Where what runs now goes during the swap: inside the staging folder for
/// a bundle (removed with it), and `<exe>.old` beside an executable, which
/// Windows only lets go of once the old Kanso has quit.
pub fn backup_path(target: &Path, stage_dir: &Path, platform: Platform) -> PathBuf {
    if platform.is_bundle() {
        stage_dir.join("previous.app")
    } else {
        let mut name = target.file_name().unwrap_or_default().to_os_string();
        name.push(".old");
        target.with_file_name(name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyError {
    #[error("the staged update is for {staged}, not this install")]
    WrongTarget { staged: PathBuf },
    #[error("the staged update is gone")]
    Missing,
    #[error("couldn't swap the update in: {0}")]
    Io(#[from] io::Error),
}

/// Swaps `pending` in for `install.target`, and leaves the old one at
/// [`backup_path`] only if it couldn't be removed. On failure everything is
/// as it was.
pub fn apply(pending: &Pending, install: &Install) -> Result<(), ApplyError> {
    if pending.target != install.target {
        return Err(ApplyError::WrongTarget {
            staged: pending.target.clone(),
        });
    }
    if !pending.payload.starts_with(&install.stage_dir) || !pending.payload.exists() {
        return Err(ApplyError::Missing);
    }
    let backup = backup_path(&install.target, &install.stage_dir, install.platform);
    remove_any(&backup)?;
    swap_in(&pending.payload, &install.target, &backup)?;
    if install.platform.is_bundle() {
        fs::remove_dir_all(&install.stage_dir).ok();
    } else {
        // Windows keeps the running executable: it goes on the next start.
        remove_any(&backup).ok();
        fs::remove_dir_all(&install.stage_dir).ok();
    }
    Ok(())
}

/// The two renames: `target` to `backup`, then `staged` to `target`,
/// undoing the first if the second fails.
pub fn swap_in(staged: &Path, target: &Path, backup: &Path) -> io::Result<()> {
    fs::rename(target, backup)?;
    if let Err(err) = fs::rename(staged, target) {
        // Put the running copy back; if even that fails the error says so.
        if let Err(undo) = fs::rename(backup, target) {
            return Err(io::Error::other(format!(
                "{err}; and putting Kanso back failed: {undo}"
            )));
        }
        return Err(err);
    }
    Ok(())
}

/// After a start: removes what an earlier update left (an `.old`
/// executable, a staging folder whose update is this version or older).
pub fn clean_up(install: &Install, current: &semver::Version) {
    let backup = backup_path(&install.target, &install.stage_dir, install.platform);
    if !install.platform.is_bundle() {
        remove_any(&backup).ok();
    }
    let stale = match Pending::load(&install.stage_dir) {
        Some(pending) => semver::Version::parse(&pending.version).map_or(true, |v| v <= *current),
        None => true,
    };
    if stale {
        fs::remove_dir_all(&install.stage_dir).ok();
    }
}

fn remove_any(path: &Path) -> io::Result<()> {
    let result = match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    };
    match result {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// Starts the installed Kanso again, telling it to wait for this one to
/// let go of the data folder: `open -n` on macOS, the executable itself
/// elsewhere. `args` are this run's own (a `--data-dir`, say).
pub fn relaunch(install: &Install, args: &[String]) -> io::Result<()> {
    let mut command = if install.platform.is_bundle() {
        let mut c = std::process::Command::new("/usr/bin/open");
        c.arg("-n").arg(&install.target).arg("--args");
        c
    } else {
        std::process::Command::new(&install.target)
    };
    command.arg("--after-update").args(args);
    command.spawn().map(drop)
}

/// The Team ID in `codesign -dv` output, unless it is "not set".
pub fn parse_team_id(codesign_output: &str) -> Option<String> {
    codesign_output
        .lines()
        .find_map(|line| line.trim().strip_prefix("TeamIdentifier="))
        .map(str::trim)
        .filter(|id| !id.is_empty() && *id != "not set")
        .map(str::to_owned)
}

/// macOS's own checks on a staged `.app`: its code signature verifies
/// (`codesign --verify --deep --strict`), and when the running Kanso is
/// signed by a developer, the update is signed by the same Team ID.
#[cfg(target_os = "macos")]
pub fn verify_bundle(staged: &Path, running: Option<&Path>) -> Result<(), String> {
    use std::process::Command;
    let out = Command::new("/usr/bin/codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(staged)
        .output()
        .map_err(|e| format!("couldn't run codesign: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }
    let team = |app: &Path| {
        Command::new("/usr/bin/codesign")
            .args(["-dv", "--verbose=2"])
            .arg(app)
            .output()
            .ok()
            .and_then(|o| parse_team_id(&String::from_utf8_lossy(&o.stderr)))
    };
    if let Some(expected) = running.and_then(team) {
        match team(staged) {
            Some(got) if got == expected => {}
            got => {
                return Err(format!(
                    "signed by {} instead of {expected}",
                    got.as_deref().unwrap_or("no developer")
                ));
            }
        }
    }
    Ok(())
}

/// Removes the quarantine flag macOS may have put on a download, so the
/// new Kanso opens without asking.
#[cfg(target_os = "macos")]
pub fn remove_quarantine(app: &Path) {
    std::process::Command::new("/usr/bin/xattr")
        .args(["-dr", "com.apple.quarantine"])
        .arg(app)
        .output()
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn opts() -> LocateOptions {
        LocateOptions::default()
    }

    #[test]
    fn a_bundle_in_a_writable_folder_installs() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("Kanso.app/Contents/MacOS/kanso");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"x").unwrap();
        let install = locate(&exe, Some(Platform::MacosArm64), opts()).unwrap();
        assert_eq!(install.target, dir.path().join("Kanso.app"));
        assert_eq!(install.stage_dir, dir.path().join(STAGE_DIR_NAME));
        let install = locate(&dir.path().join("kanso"), Some(Platform::LinuxX64), opts()).unwrap();
        assert_eq!(install.target, dir.path().join("kanso"));
    }

    #[test]
    fn each_reason_not_to_install_is_named() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("kanso");
        let mac = Some(Platform::MacosArm64);
        assert_eq!(
            locate(
                &exe,
                mac,
                LocateOptions {
                    dev_flavour: true,
                    ..opts()
                }
            ),
            Err(NotInstallable::DevFlavour)
        );
        assert_eq!(
            locate(
                &exe,
                mac,
                LocateOptions {
                    debug_build: true,
                    ..opts()
                }
            ),
            Err(NotInstallable::DevBuild)
        );
        assert_eq!(
            locate(&exe, None, opts()),
            Err(NotInstallable::UnsupportedSystem)
        );
        assert_eq!(
            locate(
                Path::new("/src/kanso/target/release/Kanso.app/Contents/MacOS/kanso"),
                mac,
                opts()
            ),
            Err(NotInstallable::CargoBuild)
        );
        assert_eq!(locate(&exe, mac, opts()), Err(NotInstallable::NotInBundle));
        assert_eq!(
            locate(
                Path::new(
                    "/private/var/folders/x/AppTranslocation/ABC/d/Kanso.app/Contents/MacOS/kanso"
                ),
                mac,
                opts()
            ),
            Err(NotInstallable::Translocated)
        );
        assert_eq!(
            locate(
                &exe,
                Some(Platform::LinuxX64),
                LocateOptions {
                    appimage: true,
                    ..opts()
                }
            ),
            Err(NotInstallable::AppImage)
        );
        let missing = dir.path().join("no/such/folder/kanso");
        assert_eq!(
            locate(&missing, Some(Platform::LinuxX64), opts()),
            Err(NotInstallable::NotWritable(
                dir.path().join("no/such/folder")
            ))
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let locked = dir.path().join("locked");
            fs::create_dir(&locked).unwrap();
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
            // Root can write anywhere; the check only means something
            // for an ordinary user.
            if !writable_dir(&locked) {
                assert_eq!(
                    locate(&locked.join("kanso"), Some(Platform::LinuxX64), opts()),
                    Err(NotInstallable::NotWritable(locked.clone()))
                );
            }
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        }
        for reason in [
            NotInstallable::DevFlavour,
            NotInstallable::NotWritable("/x".into()),
        ] {
            assert!(reason.reason().ends_with('.'));
        }
    }

    fn fake_install(dir: &Path, platform: Platform) -> (Install, Pending) {
        let (target, payload) = if platform.is_bundle() {
            let target = dir.join("Kanso.app");
            fs::create_dir_all(target.join("Contents")).unwrap();
            fs::write(target.join("Contents/version"), "old").unwrap();
            let payload = dir.join(STAGE_DIR_NAME).join("unpacked-0.2.0/Kanso.app");
            fs::create_dir_all(payload.join("Contents")).unwrap();
            fs::write(payload.join("Contents/version"), "new").unwrap();
            (target, payload)
        } else {
            let target = dir.join("kanso");
            fs::write(&target, "old").unwrap();
            let payload = dir.join(STAGE_DIR_NAME).join("unpacked-0.2.0/kanso");
            fs::create_dir_all(payload.parent().unwrap()).unwrap();
            fs::write(&payload, "new").unwrap();
            (target, payload)
        };
        let install = Install {
            target: target.clone(),
            stage_dir: dir.join(STAGE_DIR_NAME),
            platform,
        };
        let pending = Pending {
            version: "0.2.0".into(),
            payload,
            target,
            pub_date: String::new(),
            release_url: String::new(),
            notes: String::new(),
        };
        (install, pending)
    }

    fn installed(install: &Install) -> String {
        if install.platform.is_bundle() {
            fs::read_to_string(install.target.join("Contents/version")).unwrap()
        } else {
            fs::read_to_string(&install.target).unwrap()
        }
    }

    #[test]
    fn applying_swaps_and_cleans_up() {
        for platform in Platform::ALL {
            let dir = tempfile::tempdir().unwrap();
            let (install, pending) = fake_install(dir.path(), platform);
            apply(&pending, &install).unwrap();
            assert_eq!(installed(&install), "new", "{platform:?}");
            assert!(!install.stage_dir.exists());
            assert!(!backup_path(&install.target, &install.stage_dir, platform).exists());
        }
    }

    #[test]
    fn a_pending_update_for_elsewhere_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let (install, mut pending) = fake_install(dir.path(), Platform::MacosArm64);
        pending.target = "/Applications/Kanso.app".into();
        assert!(matches!(
            apply(&pending, &install),
            Err(ApplyError::WrongTarget { .. })
        ));
        let (install, mut pending) = fake_install(dir.path(), Platform::LinuxX64);
        pending.payload = dir.path().join("elsewhere");
        assert!(matches!(
            apply(&pending, &install),
            Err(ApplyError::Missing)
        ));
        assert_eq!(installed(&install), "old");
    }

    #[test]
    fn a_failed_swap_puts_the_old_one_back() {
        let dir = tempfile::tempdir().unwrap();
        let (install, pending) = fake_install(dir.path(), Platform::LinuxX64);
        let backup = backup_path(&install.target, &install.stage_dir, install.platform);
        // The staged file vanished between the check and the rename.
        let missing = dir.path().join("gone");
        assert!(swap_in(&missing, &install.target, &backup).is_err());
        assert_eq!(installed(&install), "old");
        assert!(!backup.exists());
        assert!(pending.payload.exists());
    }

    #[test]
    fn clean_up_removes_leftovers_but_keeps_a_newer_update() {
        let dir = tempfile::tempdir().unwrap();
        let (install, pending) = fake_install(dir.path(), Platform::WindowsX64);
        pending.save(&install.stage_dir).unwrap();
        let old = backup_path(&install.target, &install.stage_dir, install.platform);
        fs::write(&old, "previous").unwrap();
        clean_up(&install, &semver::Version::new(0, 1, 0));
        assert!(!old.exists(), "the .old executable goes");
        assert!(install.stage_dir.exists(), "0.2.0 is still to install");
        clean_up(&install, &semver::Version::new(0, 2, 0));
        assert!(!install.stage_dir.exists(), "0.2.0 is running now");
    }

    #[test]
    fn team_ids_are_read_from_codesign() {
        let out = "Executable=/Applications/Kanso.app/Contents/MacOS/kanso\nAuthority=Developer ID Application: Ngonidzashe Mangudya (4973Y692JU)\nTeamIdentifier=4973Y692JU\n";
        assert_eq!(parse_team_id(out).as_deref(), Some("4973Y692JU"));
        assert_eq!(
            parse_team_id("Signature=adhoc\nTeamIdentifier=not set\n"),
            None
        );
        assert_eq!(parse_team_id(""), None);
    }

    proptest! {
        /// Every swap ends with exactly one of the two copies installed and
        /// nothing lost: the new one when the staged copy was there, the
        /// old one when it wasn't.
        #[test]
        fn prop_swaps_never_lose_kanso(staged_exists in any::<bool>(), bundle in any::<bool>(), backup_left in any::<bool>()) {
            let dir = tempfile::tempdir().unwrap();
            let platform = if bundle { Platform::MacosArm64 } else { Platform::LinuxArm64 };
            let (install, pending) = fake_install(dir.path(), platform);
            let backup = backup_path(&install.target, &install.stage_dir, platform);
            if backup_left {
                // An earlier run's leftovers.
                if bundle { fs::create_dir_all(&backup).unwrap() } else { fs::write(&backup, "older").unwrap() }
            }
            if !staged_exists {
                remove_any(&pending.payload).unwrap();
            }
            let result = apply(&pending, &install);
            prop_assert_eq!(result.is_ok(), staged_exists);
            prop_assert_eq!(installed(&install), if staged_exists { "new" } else { "old" });
        }

        /// Only Cargo's own output folders count as a cargo build.
        #[test]
        fn prop_cargo_builds_are_recognised(
            prefix in prop::collection::vec("[a-s]{1,8}", 0..4),
            profile in prop::sample::select(vec!["debug", "release", "Applications", "bin"]),
            triple in any::<bool>(),
        ) {
            let mut path = PathBuf::from("/");
            for p in &prefix { path.push(p); }
            path.push("target");
            if triple { path.push("aarch64-apple-darwin"); }
            path.push(profile);
            path.push("kanso");
            let expect = profile == "debug" || profile == "release";
            prop_assert_eq!(is_cargo_build(&path), expect);
            // Without a target folder, never.
            let plain: PathBuf = ["/", "Applications", profile, "kanso"].iter().collect();
            prop_assert!(!is_cargo_build(&plain));
        }

        /// Team IDs come back exactly, whatever surrounds them.
        #[test]
        fn prop_team_ids_round_trip(id in "[A-Z0-9]{10}", before in "[a-z/ ]{0,30}") {
            let out = format!("{before}\nTeamIdentifier={id}\nSealed Resources version=2\n");
            prop_assert_eq!(parse_team_id(&out), Some(id));
        }
    }
}
