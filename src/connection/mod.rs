//! # Connection Module
//!
//! High-level connection management with transaction support.
//!
//! This module provides the `Connection` struct which wraps a driver connection
//! and adds features like:
//! - Transaction nesting via savepoints
//! - Automatic rollback on drop
//! - Transactional closure API
//! - Isolation level management

mod connection;
mod transaction;

pub use connection::Connection;
pub use transaction::TransactionGuard;
