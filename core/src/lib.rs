//! Core domain types for Pylot.
//!
//! This crate provides the foundational error type ([`PylotError`]) and the
//! virtual environment lifecycle traits ([`Create`], [`Delete`], [`Activate`])
//! that are implemented by the concrete types in `pylot-shared`.
//!
//! It intentionally has no dependency on any I/O or infrastructure concern so
//! that it can be reused by every crate in the workspace without pulling in
//! heavy transitive dependencies.

pub mod error;
pub mod venvtraits;

pub use error::{PylotError, Result};
pub use venvtraits::{Activate, Create, Delete};
