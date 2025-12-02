//! SQLite driver implementation

use async_trait::async_trait;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;
use std::str::FromStr;

use crate::core::{ConnectionError, ConnectionParams, Result};
use crate::driver::Driver;

use super::SqliteConnection;

/// SQLite database driver
#[derive(Debug, Default)]
pub struct SqliteDriver;

impl SqliteDriver {
    /// Create a new SQLite driver instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Driver for SqliteDriver {
    type Connection = SqliteConnection;

    async fn connect(&self, params: &ConnectionParams) -> Result<Self::Connection> {
        // Build connection options
        let path = params.path.as_deref().unwrap_or(":memory:");

        let options = if path == ":memory:" {
            SqliteConnectOptions::from_str("sqlite::memory:")
                .map_err(|e| ConnectionError::InvalidUrl(e.to_string()))?
        } else {
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
        };

        // Apply additional options
        let options = options
            .disable_statement_logging()
            .clone();

        // Create a single connection (not a pool) for proper transaction support
        let conn = options
            .connect()
            .await
            .map_err(|e| ConnectionError::Refused(e.to_string()))?;

        Ok(SqliteConnection::new(conn))
    }

    fn name(&self) -> &'static str {
        "sqlite"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_memory() {
        let driver = SqliteDriver::new();
        let params = ConnectionParams::sqlite_memory();

        let conn = driver.connect(&params).await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    async fn test_driver_name() {
        let driver = SqliteDriver::new();
        assert_eq!(driver.name(), "sqlite");
    }
}
