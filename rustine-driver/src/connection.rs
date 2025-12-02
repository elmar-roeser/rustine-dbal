//! Driver connection trait

use async_trait::async_trait;
use rustine_core::Result;

use crate::{DriverStatement, DriverResult};

/// A connection to a database
#[async_trait]
pub trait DriverConnection: Send + Sync {
    /// The statement type for this connection
    type Statement: DriverStatement;

    /// The result type for this connection
    type Result: DriverResult;

    /// Prepare a SQL statement
    async fn prepare(&self, sql: &str) -> Result<Self::Statement>;

    /// Execute a SQL query and return results
    async fn query(&self, sql: &str) -> Result<Self::Result>;

    /// Execute a SQL statement and return affected rows
    async fn execute(&self, sql: &str) -> Result<u64>;

    /// Begin a transaction
    async fn begin_transaction(&self) -> Result<()>;

    /// Commit the current transaction
    async fn commit(&self) -> Result<()>;

    /// Rollback the current transaction
    async fn rollback(&self) -> Result<()>;

    /// Check if the connection is still alive
    async fn is_alive(&self) -> bool;

    /// Get the server version
    async fn server_version(&self) -> Result<String>;
}
