//! Shared test helpers: a deterministic, offline [`CommandRunner`] double.
//!
//! Included by multiple test crates; each uses a different subset, so unused
//! helpers are expected per-crate.
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;

use alfred_granted::runner::CommandRunner;
use anyhow::{bail, Result};

/// A single recorded invocation of [`FakeRunner::run`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub program: String,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
}

/// A [`CommandRunner`] that returns canned output keyed by program name and
/// records every call, so tests stay offline and can assert on invocations.
pub struct FakeRunner {
    responses: HashMap<String, std::result::Result<String, String>>,
    pub calls: RefCell<Vec<Call>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            calls: RefCell::new(Vec::new()),
        }
    }

    /// Make `program` return `stdout` on success.
    pub fn with_success(mut self, program: &str, stdout: &str) -> Self {
        self.responses
            .insert(program.to_owned(), Ok(stdout.to_owned()));

        self
    }

    /// Make `program` fail with `message`.
    pub fn with_failure(mut self, program: &str, message: &str) -> Self {
        self.responses
            .insert(program.to_owned(), Err(message.to_owned()));

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
}

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[&str], envs: &[(&str, &str)]) -> Result<String> {
        self.calls.borrow_mut().push(Call {
            program: program.to_owned(),
            args: args.iter().map(|s| s.to_string()).collect(),
            envs: envs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });

        match self.responses.get(program) {
            Some(Ok(stdout)) => Ok(stdout.clone()),
            Some(Err(message)) => bail!("{message}"),
            None => bail!("unexpected program: `{program}`"),
        }
    }
}
