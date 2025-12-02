//! # Driver Module
//!
//! Database driver abstractions for Rustine DBAL.
//!
//! This module provides the `Driver` and `DriverConnection` traits that
//! define the interface between Rustine and underlying database clients.
//!
//! ## Available Drivers
//!
//! - `sqlite` - `SQLite` driver (requires `sqlite` feature)

#[allow(clippy::module_inception)]
pub mod driver;
pub mod connection;
pub mod statement;
pub mod result;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use driver::*;
pub use connection::*;
pub use statement::*;
pub use result::*;

#[cfg(feature = "sqlite")]
pub use sqlite::{SqliteDriver, SqliteConnection, SqliteStatement, SqliteResult};
