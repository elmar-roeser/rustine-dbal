//! # Platform Module
//!
//! SQL platform abstractions for generating platform-specific SQL.
//!
//! This module provides the `Platform` trait and implementations for
//! PostgreSQL, MySQL, and SQLite.

mod platform;
mod types;

pub use platform::*;
pub use types::*;
