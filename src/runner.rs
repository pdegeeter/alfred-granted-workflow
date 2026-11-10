//! Thin abstraction over the impure boundary: spawning external processes.
//!
//! Everything that talks to `assumego` or macOS `open` goes through
//! [`CommandRunner`]. Production code uses [`SystemRunner`]; tests inject a
//! fake so the rest of the crate stays deterministic and offline.

use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};

/// Abstraction over running an external command and capturing its stdout.
pub trait CommandRunner {
    /// Run `program` with `args` and the extra environment variables `envs`,
    /// returning the captured stdout on success.
    fn run(&self, program: &str, args: &[&str], envs: &[(&str, &str)]) -> Result<String>;

    /// Like [`CommandRunner::run`], but hands every stderr line to
    /// `on_stderr_line` while the command is still running, and kills it if it
    /// has not exited after `timeout`.
    ///
    /// A command killed on timeout yields an error that downcasts to
    /// [`TimedOut`].
    fn run_streaming(
        &self,
        program: &str,
        args: &[&str],
        envs: &[(&str, &str)],
        timeout: Duration,
        on_stderr_line: &mut dyn FnMut(&str),
    ) -> Result<String>;
}

/// Error returned by [`CommandRunner::run_streaming`] when the command was
/// killed because it outlived its timeout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedOut {
    pub program: String,
    pub timeout: Duration,
}

impl fmt::Display for TimedOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` did not finish within {} seconds",
            self.program,
            self.timeout.as_secs()
        )
    }
}

impl std::error::Error for TimedOut {}

/// Real implementation backed by [`std::process::Command`].
pub struct SystemRunner;

impl SystemRunner {
    /// How often [`CommandRunner::run_streaming`] checks whether the command
    /// exited while no stderr line arrives.
    const POLL_INTERVAL: Duration = Duration::from_millis(100);

    /// How long to keep collecting stderr lines once the command exited.
    const DRAIN_GRACE: Duration = Duration::from_millis(50);
}

impl CommandRunner for SystemRunner {
    fn run(&self, program: &str, args: &[&str], envs: &[(&str, &str)]) -> Result<String> {
        let output = Command::new(program)
            .args(args)
            .envs(envs.iter().copied())
            .output()
            .with_context(|| format!("failed to spawn `{program}`"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            bail!(
                "`{program}` exited with {}: {}",
                output.status,
                stderr.trim()
            );
        }

        String::from_utf8(output.stdout).context("command stdout was not valid UTF-8")
    }

    fn run_streaming(
        &self,
        program: &str,
        args: &[&str],
        envs: &[(&str, &str)],
        timeout: Duration,
        on_stderr_line: &mut dyn FnMut(&str),
    ) -> Result<String> {
        let mut child = Command::new(program)
            .args(args)
            .envs(envs.iter().copied())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to spawn `{program}`"))?;

        let mut stdout = child.stdout.take().context("child stdout was not piped")?;
        let stderr = child.stderr.take().context("child stderr was not piped")?;

        // Both pipes are read on their own threads and handed back through
        // channels, so every wait below is bounded by the deadline. The end of
        // the command is detected with `try_wait`, not with the pipes closing:
        // a process it starts (granted launches the SSO browser with its own
        // stderr) can keep them open long after it exited.
        let (stdout_sender, stdout_receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut buffer = Vec::new();
            let result = stdout.read_to_end(&mut buffer).map(|_| buffer);

            let _ = stdout_sender.send(result);
        });

        let (line_sender, line_receiver) = mpsc::channel::<String>();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                let Ok(line) = line else {
                    break;
                };

                if line_sender.send(line).is_err() {
                    break;
                }
            }
        });

        let deadline = Instant::now() + timeout;
        let mut stderr_text = String::new();
        let mut on_line = |line: String| {
            on_stderr_line(&line);
            stderr_text.push_str(&line);
            stderr_text.push('\n');
        };

        let status = loop {
            if let Some(status) = child
                .try_wait()
                .with_context(|| format!("failed to wait for `{program}`"))?
            {
                break status;
            }

            let remaining = deadline.saturating_duration_since(Instant::now());

            if remaining.is_zero() {
                let _ = child.kill();
                let _ = child.wait();

                return Err(anyhow!(TimedOut {
                    program: program.to_owned(),
                    timeout,
                }));
            }

            match line_receiver.recv_timeout(SystemRunner::POLL_INTERVAL.min(remaining)) {
                Ok(line) => on_line(line),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    thread::sleep(SystemRunner::POLL_INTERVAL.min(remaining));
                }
            }
        };

        // Lines written just before exiting may still be in flight.
        while Instant::now() < deadline {
            match line_receiver.recv_timeout(SystemRunner::DRAIN_GRACE) {
                Ok(line) => on_line(line),
                Err(_) => break,
            }
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        let stdout = stdout_receiver
            .recv_timeout(remaining.max(SystemRunner::DRAIN_GRACE))
            .map_err(|_| anyhow!("`{program}` exited but its stdout was never closed"))?
            .with_context(|| format!("failed to read the stdout of `{program}`"))?;

        if !status.success() {
            bail!("`{program}` exited with {status}: {}", stderr_text.trim());
        }

        String::from_utf8(stdout).context("command stdout was not valid UTF-8")
    }
}
