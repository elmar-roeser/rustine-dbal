//! Driver statement trait

use async_trait::async_trait;
use crate::core::{Result, SqlValue};

use super::DriverResult;

/// A prepared statement
#[async_trait]
pub trait DriverStatement: Send + Sync {
    /// The result type for this statement
    type Result: DriverResult;

    /// Bind a parameter by position (0-indexed)
    ///
    /// # Errors
    ///
    /// Returns an error if the position is invalid or binding fails.
    fn bind(&mut self, position: usize, value: SqlValue) -> Result<()>;

    /// Bind a parameter by name
    ///
    /// # Errors
    ///
    /// Returns an error if the name is not found or binding fails.
    fn bind_named(&mut self, name: &str, value: SqlValue) -> Result<()>;

    /// Execute the statement and return results
    async fn execute(&self) -> Result<Self::Result>;

    /// Execute the statement and return affected rows
    async fn execute_update(&self) -> Result<u64>;

    /// Get the SQL for this statement
    fn sql(&self) -> &str;
}
