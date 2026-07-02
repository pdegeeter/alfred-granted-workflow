//! Thin abstraction over the impure boundary: spawning external processes.
//!
//! Everything that talks to `assumego` or macOS `open` goes through
//! [`CommandRunner`]. Production code uses [`SystemRunner`]; tests inject a
//! fake so the rest of the crate stays deterministic and offline.

use std::process::Command;

use anyhow::{bail, Context, Result};

/// Abstraction over running an external command and capturing its stdout.
pub trait CommandRunner {
    /// Run `program` with `args` and the extra environment variables `envs`,
    /// returning the captured stdout on success.
    fn run(&self, program: &str, args: &[&str], envs: &[(&str, &str)]) -> Result<String>;
}

/// Real implementation backed by [`std::process::Command`].
pub struct SystemRunner;

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
}
