//! SQLite connection implementation

use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::{Error, QueryError, Result, SqlValue, TransactionError};
use crate::driver::DriverConnection;

use super::{SqliteResult, SqliteStatement};

/// SQLite database connection
pub struct SqliteConnection {
    pool: SqlitePool,
    in_transaction: AtomicBool,
}

impl SqliteConnection {
    /// Create a new SQLite connection from a pool
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            in_transaction: AtomicBool::new(false),
        }
    }

    /// Get a reference to the underlying pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Convert sqlx row to Vec<SqlValue>
    fn row_to_values(row: &sqlx::sqlite::SqliteRow) -> Vec<SqlValue> {
        use sqlx::Column;

        let columns = row.columns();
        let mut values = Vec::with_capacity(columns.len());

        for (i, col) in columns.iter().enumerate() {
            let type_info = col.type_info();
            let type_name = type_info.to_string().to_uppercase();

            let value: SqlValue = match type_name.as_str() {
                "INTEGER" | "INT" | "BIGINT" => {
                    match row.try_get::<i64, _>(i) {
                        Ok(v) => SqlValue::I64(v),
                        Err(_) => SqlValue::Null,
                    }
                }
                "REAL" | "DOUBLE" | "FLOAT" => {
                    match row.try_get::<f64, _>(i) {
                        Ok(v) => SqlValue::F64(v),
                        Err(_) => SqlValue::Null,
                    }
                }
                "TEXT" | "VARCHAR" | "CHAR" => {
                    match row.try_get::<String, _>(i) {
                        Ok(v) => SqlValue::String(v),
                        Err(_) => SqlValue::Null,
                    }
                }
                "BLOB" => {
                    match row.try_get::<Vec<u8>, _>(i) {
                        Ok(v) => SqlValue::Bytes(v),
                        Err(_) => SqlValue::Null,
                    }
                }
                "BOOLEAN" | "BOOL" => {
                    match row.try_get::<bool, _>(i) {
                        Ok(v) => SqlValue::Bool(v),
                        Err(_) => SqlValue::Null,
                    }
                }
                "NULL" => SqlValue::Null,
                _ => {
                    // Try to get as string by default
                    match row.try_get::<String, _>(i) {
                        Ok(v) => SqlValue::String(v),
                        Err(_) => SqlValue::Null,
                    }
                }
            };
            values.push(value);
        }

        values
    }

    /// Extract column names from rows
    fn extract_column_names(row: &sqlx::sqlite::SqliteRow) -> Vec<String> {
        use sqlx::Column;
        row.columns().iter().map(|c| c.name().to_string()).collect()
    }
}

// Manual implementation of Send + Sync for SqliteConnection
// sqlx::SqlitePool is already Send + Sync, and AtomicBool is too
unsafe impl Send for SqliteConnection {}
unsafe impl Sync for SqliteConnection {}

#[async_trait]
impl DriverConnection for SqliteConnection {
    type Statement = SqliteStatement;
    type Result = SqliteResult;

    async fn prepare(&self, sql: &str) -> Result<Self::Statement> {
        Ok(SqliteStatement::new(self.pool.clone(), sql.to_string()))
    }

    async fn query(&self, sql: &str) -> Result<Self::Result> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                QueryError::ExecutionFailed {
                    message: e.to_string(),
                    sql: Some(sql.to_string()),
                }
            })?;

        if rows.is_empty() {
            return Ok(SqliteResult::new(Vec::new(), Vec::new(), 0));
        }

        // Extract column names from first row
        let column_names = Self::extract_column_names(&rows[0]);

        // Convert rows
        let data: Vec<Vec<SqlValue>> = rows
            .iter()
            .map(Self::row_to_values)
            .collect();

        Ok(SqliteResult::new(data, column_names, 0))
    }

    async fn execute(&self, sql: &str) -> Result<u64> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                QueryError::ExecutionFailed {
                    message: e.to_string(),
                    sql: Some(sql.to_string()),
                }
            })?;

        Ok(result.rows_affected())
    }

    async fn begin_transaction(&self) -> Result<()> {
        if self.in_transaction.load(Ordering::SeqCst) {
            return Err(Error::Transaction(TransactionError::AlreadyActive));
        }

        sqlx::query("BEGIN TRANSACTION")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                QueryError::ExecutionFailed {
                    message: e.to_string(),
                    sql: Some("BEGIN TRANSACTION".to_string()),
                }
            })?;

        self.in_transaction.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn commit(&self) -> Result<()> {
        if !self.in_transaction.load(Ordering::SeqCst) {
            return Err(Error::Transaction(TransactionError::NoActiveTransaction));
        }

        sqlx::query("COMMIT")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                TransactionError::CommitFailed(e.to_string())
            })?;

        self.in_transaction.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn rollback(&self) -> Result<()> {
        if !self.in_transaction.load(Ordering::SeqCst) {
            return Err(Error::Transaction(TransactionError::NoActiveTransaction));
        }

        sqlx::query("ROLLBACK")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                TransactionError::RollbackFailed(e.to_string())
            })?;

        self.in_transaction.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn is_alive(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    async fn server_version(&self) -> Result<String> {
        let row = sqlx::query("SELECT sqlite_version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                QueryError::ExecutionFailed {
                    message: e.to_string(),
                    sql: Some("SELECT sqlite_version()".to_string()),
                }
            })?;

        let version: String = row.try_get(0).map_err(|e| {
            Error::conversion("SqliteRow", "String", e.to_string())
        })?;

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{Driver, DriverResult};
    use super::super::SqliteDriver;

    #[tokio::test]
    async fn test_query_basic() {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        let conn = driver.connect(&params).await.unwrap();

        let mut result = conn.query("SELECT 1 as num, 'hello' as msg").await.unwrap();
        let rows = result.all_rows().unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 2);
    }

    #[tokio::test]
    async fn test_execute_create_table() {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        let conn = driver.connect(&params).await.unwrap();

        let affected = conn.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)"
        ).await.unwrap();

        assert_eq!(affected, 0); // CREATE TABLE returns 0 affected rows
    }

    #[tokio::test]
    async fn test_is_alive() {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        let conn = driver.connect(&params).await.unwrap();

        assert!(conn.is_alive().await);
    }

    #[tokio::test]
    async fn test_server_version() {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        let conn = driver.connect(&params).await.unwrap();

        let version = conn.server_version().await.unwrap();
        assert!(!version.is_empty());
        // SQLite version typically starts with "3."
        assert!(version.starts_with("3."));
    }
}
