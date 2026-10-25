//! Starting an agent: the command, its private working folder, its scrubbed
//! environment, and a process tree Kanso can always take down.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::AcpError;

/// What to run, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentSpec {
    /// What the person calls it ("Claude Agent"), for messages.
    pub label: String,
    /// An executable, never a shell string.
    pub command: PathBuf,
    pub args: Vec<String>,
    /// The whole environment, already allowlisted (`env::agent_env`).
    pub env: Vec<(String, String)>,
    /// The empty private folder the agent works in.
    pub workspace: PathBuf,
}

impl AgentSpec {
    /// Whether the command fetches its package on first run, which is slow.
    pub fn is_npx(&self) -> bool {
        self.command
            .file_stem()
            .is_some_and(|stem| stem.eq_ignore_ascii_case("npx"))
    }
}

/// The note left in the agent's working folder.
const WORKSPACE_README: &str = "Kanso starts the AI agent you chose in this empty folder, so it \
doesn't start inside your notes or your home folder. Kanso puts nothing else here. This is not \
a sandbox: the agent runs as you.\n";

/// Makes `dir` if needed, owner-only, with only its README in it.
pub fn prepare_workspace(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    let readme = dir.join("README.txt");
    if !readme.exists() {
        std::fs::write(readme, WORKSPACE_README)?;
    }
    Ok(())
}

/// The agent's process, killed with everything it started when dropped.
pub(crate) struct ChildGuard {
    child: Child,
    /// Already killed and reaped: its pid may belong to someone else now.
    killed: bool,
    #[cfg(windows)]
    job: Option<windows::Job>,
}

impl ChildGuard {
    /// Takes down the whole tree: the process group on Unix, the Job Object
    /// on Windows, and the child itself everywhere.
    pub(crate) fn kill(&mut self) {
        if self.killed {
            return;
        }
        self.killed = true;
        #[cfg(unix)]
        {
            if let Ok(pid) = i32::try_from(self.child.id()) {
                // The child leads its own group (`process_group(0)`), so this
                // reaches `npx` → `node` → the agent. ESRCH means it's gone.
                unsafe {
                    libc::kill(-pid, libc::SIGKILL);
                }
            }
        }
        #[cfg(windows)]
        {
            if let Some(job) = &self.job {
                job.terminate();
            }
        }
        self.child.kill().ok();
        self.child.wait().ok();
    }

    /// The exit code once the process has ended, waiting up to `wait`.
    pub(crate) fn exit_code(&mut self, wait: Duration) -> Option<i32> {
        let until = Instant::now() + wait;
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => return status.code(),
                Ok(None) if Instant::now() < until => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                _ => return None,
            }
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill();
    }
}

/// Starts `spec` with piped stdio, in its workspace, with only its own
/// environment. No shell is involved.
pub(crate) fn spawn(
    spec: &AgentSpec,
) -> Result<(ChildGuard, ChildStdin, ChildStdout, ChildStderr), AcpError> {
    prepare_workspace(&spec.workspace)
        .map_err(|err| AcpError::Spawn(format!("Couldn't make its working folder: {err}")))?;
    let (program, args) = resolve_program(&spec.command, &spec.args, &spec.env);
    let mut command = Command::new(&program);
    command
        .args(&args)
        .current_dir(&spec.workspace)
        .env_clear()
        .envs(spec.env.iter().map(|(name, value)| (name, value)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command
        .spawn()
        .map_err(|err| AcpError::Spawn(err.to_string()))?;
    let (Some(stdin), Some(stdout), Some(stderr)) =
        (child.stdin.take(), child.stdout.take(), child.stderr.take())
    else {
        child.kill().ok();
        return Err(AcpError::Spawn(
            "Its input and output weren't connected.".to_owned(),
        ));
    };
    let guard = ChildGuard {
        #[cfg(windows)]
        job: windows::Job::assign(&child),
        child,
        killed: false,
    };
    Ok((guard, stdin, stdout, stderr))
}

/// The program and arguments to run for `command`. On Windows an npm `.cmd`
/// shim can't be started without `cmd.exe`, so it becomes `node <script>`.
fn resolve_program(
    command: &Path,
    args: &[String],
    env: &[(String, String)],
) -> (PathBuf, Vec<String>) {
    let is_shim = command
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat"));
    if is_shim
        && let Ok(text) = std::fs::read_to_string(command)
        && let Some(script) = shim_script(&text)
        && let Some(dir) = command.parent()
    {
        let path = env
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("PATH"))
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        let node =
            crate::detect::which_in("node", &std::env::split_paths(&path).collect::<Vec<_>>())
                .unwrap_or_else(|| PathBuf::from("node"));
        let mut full = vec![dir.join(script).to_string_lossy().into_owned()];
        full.extend(args.iter().cloned());
        return (node, full);
    }
    (command.to_path_buf(), args.to_vec())
}

/// The script an npm `.cmd` shim runs, relative to the shim's folder:
/// `"%dp0%\node_modules\pkg\bin\cli.js" %*` gives `node_modules\pkg\bin\cli.js`.
pub fn shim_script(text: &str) -> Option<String> {
    for marker in ["%dp0%\\", "%~dp0\\"] {
        let mut rest = text;
        while let Some(start) = rest.find(marker) {
            let after = &rest[start + marker.len()..];
            let end = after.find(['"', '\r', '\n']).unwrap_or(after.len());
            let candidate = after[..end].trim();
            let lower = candidate.to_ascii_lowercase();
            if [".js", ".mjs", ".cjs"]
                .iter()
                .any(|ext| lower.ends_with(ext))
            {
                return Some(candidate.replace('\\', std::path::MAIN_SEPARATOR_STR));
            }
            rest = after;
        }
    }
    None
}

/// The last of what the agent wrote to stderr, at most `cap` bytes, with
/// anything that looks like a token taken out. Shown only when an agent
/// doesn't start.
#[derive(Debug, Default)]
pub(crate) struct StderrRing {
    text: String,
    cap: usize,
}

impl StderrRing {
    pub(crate) fn new(cap: usize) -> Self {
        StderrRing {
            text: String::new(),
            cap,
        }
    }

    pub(crate) fn push(&mut self, chunk: &str) {
        self.text.push_str(&redact(chunk));
        if self.text.len() > self.cap {
            let mut cut = self.text.len() - self.cap;
            while !self.text.is_char_boundary(cut) {
                cut += 1;
            }
            self.text.drain(..cut);
        }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}

/// `text` with every long run of token characters replaced.
pub fn redact(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if run.chars().count() >= 24 {
            out.push_str("[redacted]");
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '+' | '/' | '=') {
            run.push(ch);
        } else {
            flush(&mut run, &mut out);
            out.push(ch);
        }
    }
    flush(&mut run, &mut out);
    out
}

/// Reads `stderr` into `ring` until it closes.
pub(crate) fn drain_stderr(mut stderr: ChildStderr, ring: Arc<Mutex<StderrRing>>) {
    std::thread::Builder::new()
        .name("kanso-acp-stderr".into())
        .spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(read) = stderr.read(&mut buf) {
                if read == 0 {
                    break;
                }
                let chunk = String::from_utf8_lossy(&buf[..read]).into_owned();
                if let Ok(mut ring) = ring.lock() {
                    ring.push(&chunk);
                }
            }
        })
        .ok();
}

#[cfg(windows)]
mod windows {
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject,
    };

    /// A Job Object holding the agent: closing it kills every process in it.
    pub(super) struct Job(HANDLE);

    // The handle is only used to terminate and close the job.
    unsafe impl Send for Job {}
    unsafe impl Sync for Job {}

    impl Job {
        pub(super) fn assign(child: &std::process::Child) -> Option<Job> {
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return None;
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let set = SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    (&raw const info).cast(),
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                );
                if set == 0 || AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) == 0 {
                    CloseHandle(job);
                    return None;
                }
                Some(Job(job))
            }
        }

        pub(super) fn terminate(&self) {
            unsafe {
                TerminateJobObject(self.0, 1);
            }
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StderrRing, redact, shim_script};
    use proptest::prelude::*;

    #[test]
    fn an_npm_shim_names_its_script() {
        let shim = "@ECHO off\r\nGOTO start\r\n:find_dp0\r\nSET dp0=%~dp0\r\nEXIT /b\r\n:start\r\nSETLOCAL\r\nCALL :find_dp0\r\n\r\nIF EXIST \"%dp0%\\node.exe\" (\r\n  SET \"_prog=%dp0%\\node.exe\"\r\n) ELSE (\r\n  SET \"_prog=node\"\r\n)\r\n\r\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & \"%_prog%\"  \"%dp0%\\node_modules\\@agentclientprotocol\\claude-agent-acp\\dist\\index.js\" %*\r\n";
        let script = shim_script(shim).unwrap();
        let sep = std::path::MAIN_SEPARATOR_STR;
        assert_eq!(
            script,
            [
                "node_modules",
                "@agentclientprotocol",
                "claude-agent-acp",
                "dist",
                "index.js"
            ]
            .join(sep)
        );
        assert_eq!(shim_script("@echo off\r\nfoo.exe %*\r\n"), None);
    }

    #[test]
    fn stderr_keeps_the_end_and_hides_tokens() {
        let mut ring = StderrRing::new(40);
        ring.push("Error: invalid key sk-ant-api03-abcdefghijklmnopqrstuvwxyz\n");
        assert!(!ring.text().contains("abcdefghijklmnop"));
        ring.push("then a much longer line that pushes the start out of the ring\n");
        assert!(ring.text().len() <= 40);
        assert!(ring.text().ends_with("ring\n"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Nothing 24 or more token characters long survives redaction, and
        /// short words are kept as they were.
        #[test]
        fn prop_redaction_leaves_no_long_token(
            words in prop::collection::vec("[A-Za-z0-9_-]{1,40}", 0..10),
        ) {
            let text = words.join(" ");
            let redacted = redact(&text);
            for word in redacted.split(' ') {
                prop_assert!(word == "[redacted]" || word.chars().count() < 24);
            }
            for word in &words {
                if word.chars().count() < 24 {
                    prop_assert!(redacted.split(' ').any(|kept| kept == word));
                }
            }
        }
    }
}
