//! Core library for the `alfred-granted` Alfred workflow.
//!
//! The binary is a thin dispatcher over these modules. Everything that can be
//! made pure (parsing, filtering, URL extraction) lives here and is unit
//! tested; the only impure boundary is [`runner::CommandRunner`], which is
//! mocked in tests.

pub mod alfred;
pub mod console;
pub mod frecency;
pub mod profiles;
pub mod runner;
