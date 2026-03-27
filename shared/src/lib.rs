//! Infrastructure and shared services for the Pylot workspace.
//!
//! `pylot-shared` collects every reusable module that both the CLI (`pylot`)
//! and the TUI (`pylot-tui`) depend on:
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`cfg::settings`] | Process-wide [`Settings`](cfg::settings::Settings) singleton loaded from `settings.toml` |
//! | [`cfg::logger`] | Colored `env_logger` initializer |
//! | [`infra::processes`] | Spawn subprocesses and activate virtual environment shells |
//! | [`uv::uvctrl`] | Install, update, uninstall, and check Astral UV |
//! | [`virtualenv::uvvenv`] | [`UvVenv`](virtualenv::uvvenv::UvVenv) — concrete virtual environment type |
//! | [`virtualenv::venvmanager`] | Discovery, selection, and table rendering for environments |
//! | [`virtualenv::venvtraits`] | Re-export of the [`Create`](virtualenv::venvtraits::Create) / [`Delete`](virtualenv::venvtraits::Delete) / [`Activate`](virtualenv::venvtraits::Activate) traits |
//! | [`utility::utils`] | Confirmation prompts, requirements-file parsing, path helpers |
//! | [`utility::constants`] | Platform constants (commands, paths, error messages) |
//! | [`error`] | Re-export of [`PylotError`] and the [`Result`] alias |

pub mod cfg;
pub mod infra;
pub mod error;
pub mod utility;
pub mod uv;
pub mod virtualenv;

pub use cfg::{logger, settings};
pub use infra::processes;
pub use error::{PylotError, Result};
pub use utility::{constants, utils};
pub use uv::uvctrl;
pub use virtualenv::{uvvenv, venvmanager, venvtraits};
