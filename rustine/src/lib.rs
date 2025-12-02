//! # Rustine
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
//! ## Quick Start
//!
//! ```rust,ignore
//! use rustine::prelude::*;
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

// Re-export core types
pub use rustine_core::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use rustine_core::prelude::*;
}
