//! Shared test helpers: a deterministic, offline [`CommandRunner`] double.
//!
//! Included by multiple test crates; each uses a different subset, so unused
//! helpers are expected per-crate.
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use alfred_granted::runner::{CommandRunner, TimedOut};
use anyhow::{anyhow, bail, Result};

/// A single recorded invocation of [`FakeRunner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub program: String,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
    /// The timeout given to `run_streaming`; `None` for a plain `run`.
    pub timeout: Option<Duration>,
}

/// Canned behaviour of one program.
#[derive(Debug, Clone)]
enum Response {
    Success(String),
    Failure(String),
    TimedOut,
}

/// A [`CommandRunner`] that returns canned output keyed by program name and
/// records every call, so tests stay offline and can assert on invocations.
pub struct FakeRunner {
    responses: HashMap<String, Response>,
    stderr: HashMap<String, Vec<String>>,
    pub calls: RefCell<Vec<Call>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            stderr: HashMap::new(),
            calls: RefCell::new(Vec::new()),
        }
    }

    /// Make `program` return `stdout` on success.
    pub fn with_success(mut self, program: &str, stdout: &str) -> Self {
        self.responses
            .insert(program.to_owned(), Response::Success(stdout.to_owned()));

        self
    }

    /// Make `program` fail with `message`.
    pub fn with_failure(mut self, program: &str, message: &str) -> Self {
        self.responses
            .insert(program.to_owned(), Response::Failure(message.to_owned()));

        self
    }

    /// Make `program` run past its timeout (`run_streaming` only).
    pub fn with_timeout(mut self, program: &str) -> Self {
        self.responses
            .insert(program.to_owned(), Response::TimedOut);

        self
    }

    /// Make `program` write `lines` to stderr before it ends (`run_streaming`
    /// only).
    pub fn with_stderr(mut self, program: &str, lines: &[&str]) -> Self {
        self.stderr.insert(
            program.to_owned(),
            lines.iter().map(|line| (*line).to_owned()).collect(),
        );

        self
    }

    /// The programs invoked so far, in order.
    pub fn programs_called(&self) -> Vec<String> {
        self.calls
            .borrow()
            .iter()
            .map(|call| call.program.clone())
            .collect()
    }

    fn record(
        &self,
        program: &str,
        args: &[&str],
        envs: &[(&str, &str)],
        timeout: Option<Duration>,
    ) {
        self.calls.borrow_mut().push(Call {
            program: program.to_owned(),
            args: args.iter().map(|s| s.to_string()).collect(),
            envs: envs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            timeout,
        });
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[&str], envs: &[(&str, &str)]) -> Result<String> {
        self.record(program, args, envs, None);

        match self.responses.get(program) {
            Some(Response::Success(stdout)) => Ok(stdout.clone()),
            Some(Response::Failure(message)) => bail!("{message}"),
            Some(Response::TimedOut) => bail!("`{program}` cannot time out without a timeout"),
            None => bail!("unexpected program: `{program}`"),
        }
    }

    fn run_streaming(
        &self,
        program: &str,
        args: &[&str],
        envs: &[(&str, &str)],
        timeout: Duration,
        on_stderr_line: &mut dyn FnMut(&str),
    ) -> Result<String> {
        self.record(program, args, envs, Some(timeout));

        // The borrow of `calls` is released above: the callback may run other
        // commands through this runner.
        for line in self.stderr.get(program).cloned().unwrap_or_default() {
            on_stderr_line(&line);
        }

        match self.responses.get(program) {
            Some(Response::Success(stdout)) => Ok(stdout.clone()),
            Some(Response::Failure(message)) => bail!("{message}"),
            Some(Response::TimedOut) => Err(anyhow!(TimedOut {
                program: program.to_owned(),
                timeout,
            })),
            None => bail!("unexpected program: `{program}`"),
        }
    }
}
