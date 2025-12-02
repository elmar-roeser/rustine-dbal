//! # Rustine DBAL
//!
//! Rustine is an idiomatic Rust Database Abstraction Layer inspired by Doctrine DBAL.
//!
//! ## Features
//!
//! - **Connection Management**: Lazy connections, auto-reconnect, connection pooling
//! - **Transaction Support**: ACID transactions with savepoint-based nesting
//! - **Query Builder**: Type-safe, fluent API for building SQL queries
//! - **Schema Introspection**: Read and manipulate database schemas
//! - **Platform Abstraction**: Write once, run on PostgreSQL, MySQL, SQLite
//! - **Type System**: Bidirectional conversion between Rust and SQL types
//!
//! ## Modules
//!
//! - [`core`] - Core types, traits, and errors
//! - [`driver`] - Database driver abstractions
//! - [`platform`] - SQL dialect implementations
//! - [`query`] - Query builder (coming soon)
//! - [`schema`] - Schema introspection (coming soon)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rustine_dbal::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Connect to PostgreSQL
//!     let conn = Connection::from_url("postgres://user:pass@localhost/db").await?;
//!
//!     // Execute a query
//!     let rows = conn.fetch_all("SELECT * FROM users WHERE active = $1", &[&true]).await?;
//!
//!     // Use transactions
//!     conn.transactional(|tx| async move {
//!         tx.execute("INSERT INTO users (name) VALUES ($1)", &[&"Alice"]).await?;
//!         Ok(())
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod connection;
pub mod driver;
pub mod platform;
pub mod query;
pub mod schema;

/// Prelude module for convenient imports
///
/// ```rust
/// use rustine_dbal::prelude::*;
/// ```
pub mod prelude {
    // Core types
    pub use crate::core::{
        Error, Result, ConnectionError, TransactionError, SchemaError, QueryError,
        SqlValue, ToSql, FromSql,
        ParameterType,
        Configuration, ConnectionParams, IsolationLevel,
    };

    // Driver traits
    pub use crate::driver::{
        Driver, DriverConnection, DriverStatement, DriverResult,
    };

    // SQLite driver (when enabled)
    #[cfg(feature = "sqlite")]
    pub use crate::driver::{SqliteDriver, SqliteConnection, SqliteStatement, SqliteResult};

    // Platform traits
    pub use crate::platform::Platform;

    // Connection
    pub use crate::connection::Connection;

    // Query Builder
    pub use crate::query::{QueryBuilder, Expr};

    // Schema
    pub use crate::schema::{SchemaManager, TableInfo, ColumnInfo};
}

// Re-export commonly used types at crate root
pub use core::{Error, Result, SqlValue, ToSql, FromSql};
pub use core::{Configuration, ConnectionParams};
