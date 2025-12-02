//! # Rustine Driver
//!
//! Database driver abstractions for Rustine DBAL.
//!
//! This crate provides the `Driver` and `DriverConnection` traits that
//! define the interface between Rustine and underlying database clients.

pub mod driver;
pub mod connection;
pub mod statement;
pub mod result;

pub use driver::*;
pub use connection::*;
pub use statement::*;
pub use result::*;
