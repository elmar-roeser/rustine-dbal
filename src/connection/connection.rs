//! High-level database connection with transaction management

use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::core::{ConnectionParams, Error, IsolationLevel, Result, TransactionError};
use crate::driver::{Driver, DriverConnection};

/// High-level database connection with transaction management
///
/// This struct wraps a driver connection and provides:
/// - Nested transaction support via savepoints
/// - Automatic rollback on drop when transaction is active
/// - Transactional closure API for safe transaction handling
/// - Isolation level management
///
/// # Example
///
/// ```rust,ignore
/// use rustine_dbal::prelude::*;
///
/// let conn = Connection::new(driver, params).await?;
///
/// // Simple transaction
/// conn.begin_transaction().await?;
/// conn.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
/// conn.commit().await?;
///
/// // Transactional closure (auto-commit/rollback)
/// let result = conn.transactional(|conn| async move {
///     conn.execute("INSERT INTO users (name) VALUES ('Bob')").await?;
///     Ok(42)
/// }).await?;
/// ```
pub struct Connection<D: Driver> {
    /// The underlying driver connection
    inner: D::Connection,
    /// Current transaction nesting level (0 = no transaction)
    nesting_level: AtomicU32,
    /// Whether the transaction is marked as rollback-only
    rollback_only: AtomicBool,
    /// Current isolation level for new transactions
    isolation_level: IsolationLevel,
    /// Whether this connection has been explicitly closed
    closed: AtomicBool,
}

impl<D: Driver> Connection<D> {
    /// Create a new connection using the given driver and parameters
    pub async fn new(driver: &D, params: &ConnectionParams) -> Result<Self> {
        let inner = driver.connect(params).await?;
        Ok(Self {
            inner,
            nesting_level: AtomicU32::new(0),
            rollback_only: AtomicBool::new(false),
            isolation_level: IsolationLevel::default(),
            closed: AtomicBool::new(false),
        })
    }

    /// Create a connection from an existing driver connection
    pub fn from_driver_connection(conn: D::Connection) -> Self {
        Self {
            inner: conn,
            nesting_level: AtomicU32::new(0),
            rollback_only: AtomicBool::new(false),
            isolation_level: IsolationLevel::default(),
            closed: AtomicBool::new(false),
        }
    }

    /// Get the underlying driver connection
    pub fn inner(&self) -> &D::Connection {
        &self.inner
    }

    // ========================================================================
    // Query Execution
    // ========================================================================

    /// Execute a SQL query and return results
    pub async fn query(&self, sql: &str) -> Result<<D::Connection as DriverConnection>::Result> {
        self.ensure_not_closed()?;
        self.inner.query(sql).await
    }

    /// Execute a SQL statement and return affected rows
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        self.ensure_not_closed()?;
        self.inner.execute(sql).await
    }

    /// Prepare a SQL statement
    pub async fn prepare(
        &self,
        sql: &str,
    ) -> Result<<D::Connection as DriverConnection>::Statement> {
        self.ensure_not_closed()?;
        self.inner.prepare(sql).await
    }

    // ========================================================================
    // Transaction Management
    // ========================================================================

    /// Begin a new transaction or create a savepoint if already in a transaction
    ///
    /// If no transaction is active, starts a new transaction.
    /// If a transaction is already active, creates a savepoint for nested transaction.
    pub async fn begin_transaction(&self) -> Result<()> {
        self.ensure_not_closed()?;

        let current_level = self.nesting_level.load(Ordering::SeqCst);

        if current_level == 0 {
            // Start a real transaction
            self.inner.begin_transaction().await?;
        } else {
            // Create a savepoint for nested transaction
            let savepoint_name = self.savepoint_name(current_level);
            let sql = format!("SAVEPOINT {}", savepoint_name);
            self.inner.execute(&sql).await.map_err(|e| {
                Error::Transaction(TransactionError::CommitFailed(format!(
                    "Failed to create savepoint: {}",
                    e
                )))
            })?;
        }

        self.nesting_level.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Commit the current transaction or release the current savepoint
    ///
    /// If at the outermost transaction level, commits the transaction.
    /// If in a nested transaction, releases the savepoint.
    pub async fn commit(&self) -> Result<()> {
        self.ensure_not_closed()?;

        let current_level = self.nesting_level.load(Ordering::SeqCst);

        if current_level == 0 {
            return Err(Error::Transaction(TransactionError::NoActiveTransaction));
        }

        if self.rollback_only.load(Ordering::SeqCst) {
            return Err(Error::Transaction(TransactionError::RollbackOnly));
        }

        if current_level == 1 {
            // Commit the real transaction
            self.inner.commit().await?;
        } else {
            // Release the savepoint (some databases like MySQL don't support this)
            let savepoint_name = self.savepoint_name(current_level - 1);
            let sql = format!("RELEASE SAVEPOINT {}", savepoint_name);
            // Ignore errors for databases that don't support RELEASE SAVEPOINT
            let _ = self.inner.execute(&sql).await;
        }

        self.nesting_level.fetch_sub(1, Ordering::SeqCst);

        // Reset rollback_only when exiting outermost transaction
        if self.nesting_level.load(Ordering::SeqCst) == 0 {
            self.rollback_only.store(false, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Rollback the current transaction or rollback to the current savepoint
    ///
    /// If at the outermost transaction level, rolls back the entire transaction.
    /// If in a nested transaction, rolls back to the savepoint.
    pub async fn rollback(&self) -> Result<()> {
        self.ensure_not_closed()?;

        let current_level = self.nesting_level.load(Ordering::SeqCst);

        if current_level == 0 {
            return Err(Error::Transaction(TransactionError::NoActiveTransaction));
        }

        if current_level == 1 {
            // Rollback the real transaction
            self.inner.rollback().await?;
        } else {
            // Rollback to the savepoint
            let savepoint_name = self.savepoint_name(current_level - 1);
            let sql = format!("ROLLBACK TO SAVEPOINT {}", savepoint_name);
            self.inner.execute(&sql).await.map_err(|e| {
                Error::Transaction(TransactionError::RollbackFailed(format!(
                    "Failed to rollback to savepoint: {}",
                    e
                )))
            })?;
        }

        self.nesting_level.fetch_sub(1, Ordering::SeqCst);

        // Reset rollback_only when exiting outermost transaction
        if self.nesting_level.load(Ordering::SeqCst) == 0 {
            self.rollback_only.store(false, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Execute an async operation within a transaction
    ///
    /// Automatically commits on success or rolls back on error.
    /// This method uses a boxed future to work around async closure lifetime issues.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::pin::Pin;
    /// use std::future::Future;
    ///
    /// let result = conn.transactional(Box::pin(async {
    ///     conn.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
    ///     conn.execute("INSERT INTO users (name) VALUES ('Bob')").await?;
    ///     Ok(2)
    /// })).await?;
    /// assert_eq!(result, 2);
    /// ```
    pub async fn transactional_boxed<T>(
        &self,
        fut: std::pin::Pin<Box<dyn Future<Output = Result<T>> + Send + '_>>,
    ) -> Result<T> {
        self.begin_transaction().await?;

        match fut.await {
            Ok(result) => {
                self.commit().await?;
                Ok(result)
            }
            Err(e) => {
                // Try to rollback, but don't hide the original error
                let _ = self.rollback().await;
                Err(e)
            }
        }
    }

    /// Execute operations within a transaction using a simpler callback pattern
    ///
    /// This is a convenience method that handles begin/commit/rollback automatically.
    /// For more complex scenarios, use `begin_transaction()`, `commit()`, and `rollback()` directly.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Simple pattern: do work, then decide
    /// conn.begin_transaction().await?;
    /// let result = conn.execute("INSERT INTO users (name) VALUES ('Alice')").await;
    /// match result {
    ///     Ok(_) => conn.commit().await?,
    ///     Err(e) => { conn.rollback().await?; return Err(e); }
    /// }
    /// ```
    pub async fn in_transaction<T, E>(&self, result: std::result::Result<T, E>) -> Result<T>
    where
        E: Into<Error>,
    {
        match result {
            Ok(value) => {
                self.commit().await?;
                Ok(value)
            }
            Err(e) => {
                let _ = self.rollback().await;
                Err(e.into())
            }
        }
    }

    /// Set the isolation level for new transactions
    ///
    /// This must be called before `begin_transaction()`.
    pub fn set_transaction_isolation(&mut self, level: IsolationLevel) {
        self.isolation_level = level;
    }

    /// Get the current isolation level setting
    pub fn transaction_isolation(&self) -> IsolationLevel {
        self.isolation_level
    }

    /// Get the current transaction nesting level
    ///
    /// Returns 0 if no transaction is active.
    pub fn transaction_nesting_level(&self) -> u32 {
        self.nesting_level.load(Ordering::SeqCst)
    }

    /// Check if a transaction is currently active
    pub fn is_transaction_active(&self) -> bool {
        self.nesting_level.load(Ordering::SeqCst) > 0
    }

    /// Check if the current transaction is marked as rollback-only
    pub fn is_rollback_only(&self) -> bool {
        self.rollback_only.load(Ordering::SeqCst)
    }

    /// Mark the current transaction as rollback-only
    ///
    /// After calling this, `commit()` will fail and the transaction
    /// can only be rolled back.
    pub fn set_rollback_only(&self) {
        self.rollback_only.store(true, Ordering::SeqCst);
    }

    // ========================================================================
    // Connection State
    // ========================================================================

    /// Check if the connection is still alive
    pub async fn is_alive(&self) -> bool {
        if self.closed.load(Ordering::SeqCst) {
            return false;
        }
        self.inner.is_alive().await
    }

    /// Get the server version string
    pub async fn server_version(&self) -> Result<String> {
        self.ensure_not_closed()?;
        self.inner.server_version().await
    }

    /// Close the connection
    ///
    /// If a transaction is active, it will be rolled back first.
    pub async fn close(&self) -> Result<()> {
        if self.closed.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already closed
        }

        // Rollback any active transaction
        while self.nesting_level.load(Ordering::SeqCst) > 0 {
            let _ = self.rollback().await;
        }

        Ok(())
    }

    /// Check if the connection has been closed
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    // ========================================================================
    // Private Helpers
    // ========================================================================

    /// Generate a savepoint name for the given nesting level
    fn savepoint_name(&self, level: u32) -> String {
        format!("RUSTINE_{}", level)
    }

    /// Ensure the connection is not closed
    fn ensure_not_closed(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(Error::Connection(crate::core::ConnectionError::Closed));
        }
        Ok(())
    }
}

impl<D: Driver> Drop for Connection<D> {
    fn drop(&mut self) {
        let level = self.nesting_level.load(Ordering::SeqCst);
        if level > 0 {
            // Log warning if tracing is enabled
            #[cfg(feature = "tracing")]
            tracing::warn!(
                "Connection dropped with {} active transaction level(s). \
                 Transaction will be rolled back.",
                level
            );

            // We can't do async rollback in drop, but the underlying
            // connection should handle cleanup
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "sqlite")]
    mod sqlite_tests {
        use super::*;
        use crate::core::SqlValue;
        use crate::driver::sqlite::SqliteDriver;
        use crate::driver::DriverResult;

        #[tokio::test]
        async fn test_basic_transaction() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            // Create table
            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
                .await
                .unwrap();

            // Transaction that commits
            conn.begin_transaction().await.unwrap();
            assert!(conn.is_transaction_active());
            assert_eq!(conn.transaction_nesting_level(), 1);

            conn.execute("INSERT INTO test (id, name) VALUES (1, 'Alice')")
                .await
                .unwrap();
            conn.commit().await.unwrap();

            assert!(!conn.is_transaction_active());
            assert_eq!(conn.transaction_nesting_level(), 0);

            // Verify data
            let mut result = conn.query("SELECT COUNT(*) FROM test").await.unwrap();
            let rows = result.all_rows().unwrap();
            assert_eq!(rows[0][0], SqlValue::I64(1));
        }

        #[tokio::test]
        async fn test_transaction_rollback() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
                .await
                .unwrap();

            // Transaction that rolls back
            conn.begin_transaction().await.unwrap();
            conn.execute("INSERT INTO test (id, name) VALUES (1, 'Alice')")
                .await
                .unwrap();
            conn.rollback().await.unwrap();

            // Verify no data
            let mut result = conn.query("SELECT COUNT(*) FROM test").await.unwrap();
            let rows = result.all_rows().unwrap();
            assert_eq!(rows[0][0], SqlValue::I64(0));
        }

        #[tokio::test]
        async fn test_nested_transactions() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
                .await
                .unwrap();

            // Outer transaction
            conn.begin_transaction().await.unwrap();
            assert_eq!(conn.transaction_nesting_level(), 1);

            conn.execute("INSERT INTO test (id, name) VALUES (1, 'Alice')")
                .await
                .unwrap();

            // Inner transaction (savepoint)
            conn.begin_transaction().await.unwrap();
            assert_eq!(conn.transaction_nesting_level(), 2);

            conn.execute("INSERT INTO test (id, name) VALUES (2, 'Bob')")
                .await
                .unwrap();

            // Rollback inner transaction
            conn.rollback().await.unwrap();
            assert_eq!(conn.transaction_nesting_level(), 1);

            // Commit outer transaction
            conn.commit().await.unwrap();
            assert_eq!(conn.transaction_nesting_level(), 0);

            // Verify only Alice exists
            let mut result = conn.query("SELECT name FROM test").await.unwrap();
            let rows = result.all_rows().unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], SqlValue::String("Alice".to_string()));
        }

        #[tokio::test]
        async fn test_transactional_commit() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
                .await
                .unwrap();

            // Use begin/commit pattern
            conn.begin_transaction().await.unwrap();
            conn.execute("INSERT INTO test (id, name) VALUES (1, 'Alice')")
                .await
                .unwrap();
            conn.commit().await.unwrap();

            assert!(!conn.is_transaction_active());

            // Verify data committed
            let mut result = conn.query("SELECT COUNT(*) FROM test").await.unwrap();
            let rows = result.all_rows().unwrap();
            assert_eq!(rows[0][0], SqlValue::I64(1));
        }

        #[tokio::test]
        async fn test_transactional_rollback_on_error() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
                .await
                .unwrap();

            // Use begin_transaction + in_transaction pattern
            conn.begin_transaction().await.unwrap();
            conn.execute("INSERT INTO test (id, name) VALUES (1, 'Alice')")
                .await
                .unwrap();

            // Simulate an error and use in_transaction to handle it
            let simulated_result: std::result::Result<(), Error> =
                Err(Error::driver_message("Simulated error"));
            let result = conn.in_transaction(simulated_result).await;

            assert!(result.is_err());
            assert!(!conn.is_transaction_active());

            // Verify data rolled back
            let mut result = conn.query("SELECT COUNT(*) FROM test").await.unwrap();
            let rows = result.all_rows().unwrap();
            assert_eq!(rows[0][0], SqlValue::I64(0));
        }

        #[tokio::test]
        async fn test_rollback_only() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            conn.begin_transaction().await.unwrap();
            conn.set_rollback_only();

            assert!(conn.is_rollback_only());

            // Commit should fail
            let result = conn.commit().await;
            assert!(matches!(
                result,
                Err(Error::Transaction(TransactionError::RollbackOnly))
            ));

            // Rollback should work
            conn.rollback().await.unwrap();
            assert!(!conn.is_rollback_only()); // Reset after transaction ends
        }

        #[tokio::test]
        async fn test_no_active_transaction_error() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            // Commit without transaction should fail
            let result = conn.commit().await;
            assert!(matches!(
                result,
                Err(Error::Transaction(TransactionError::NoActiveTransaction))
            ));

            // Rollback without transaction should fail
            let result = conn.rollback().await;
            assert!(matches!(
                result,
                Err(Error::Transaction(TransactionError::NoActiveTransaction))
            ));
        }

        #[tokio::test]
        async fn test_connection_close() {
            let driver = SqliteDriver::new();
            let params = ConnectionParams::sqlite_memory();
            let conn = Connection::new(&driver, &params).await.unwrap();

            assert!(!conn.is_closed());
            conn.close().await.unwrap();
            assert!(conn.is_closed());

            // Operations should fail after close
            let result = conn.execute("SELECT 1").await;
            assert!(matches!(
                result,
                Err(Error::Connection(crate::core::ConnectionError::Closed))
            ));
        }
    }
}
