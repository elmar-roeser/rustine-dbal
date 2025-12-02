//! # Rustine Platform
//!
//! SQL platform abstractions for generating platform-specific SQL.
//!
//! This crate provides the `Platform` trait and implementations for
//! PostgreSQL, MySQL, and SQLite.

mod platform;

pub use platform::*;
