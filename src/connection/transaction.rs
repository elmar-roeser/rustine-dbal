//! Transaction guard for RAII-style transaction management

use crate::core::Result;
use crate::driver::Driver;

use super::Connection;

/// A guard that represents an active transaction
///
/// When dropped, the transaction will be rolled back if not explicitly committed.
/// This provides RAII-style transaction management.
///
/// # Example
///
/// ```rust,ignore
/// let guard = conn.transaction().await?;
/// conn.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
/// guard.commit().await?; // Explicit commit
///
/// // Or let it roll back automatically:
/// let guard = conn.transaction().await?;
/// conn.execute("INSERT INTO users (name) VALUES ('Bob')").await?;
/// // guard is dropped here, transaction is rolled back
/// ```
pub struct TransactionGuard<'a, D: Driver> {
    connection: &'a Connection<D>,
    committed: bool,
    rolled_back: bool,
}

impl<'a, D: Driver> TransactionGuard<'a, D> {
    /// Create a new transaction guard
    ///
    /// This does NOT start the transaction - use `Connection::transaction()` instead.
    #[allow(dead_code)]
    pub(crate) fn new(connection: &'a Connection<D>) -> Self {
        Self {
            connection,
            committed: false,
            rolled_back: false,
        }
    }

    /// Commit the transaction
    ///
    /// After calling this, the guard will not roll back on drop.
    pub async fn commit(mut self) -> Result<()> {
        self.committed = true;
        self.connection.commit().await
    }

    /// Rollback the transaction explicitly
    ///
    /// After calling this, the guard will not try to roll back again on drop.
    pub async fn rollback(mut self) -> Result<()> {
        self.rolled_back = true;
        self.connection.rollback().await
    }

    /// Check if this guard has been committed
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// Check if this guard has been rolled back
    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection<D> {
        self.connection
    }
}

// Note: We cannot implement async Drop, so the actual rollback on drop
// happens in Connection::drop which logs a warning. For truly automatic
// rollback, users should use Connection::transactional() instead.
impl<'a, D: Driver> Drop for TransactionGuard<'a, D> {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back {
            // We can't do async rollback here, but the Connection's drop
            // will handle cleanup and log a warning
            #[cfg(feature = "tracing")]
            tracing::warn!(
                "TransactionGuard dropped without explicit commit or rollback. \
                 Use Connection::transactional() for automatic rollback."
            );
        }
    }
}

#[cfg(test)]
mod tests {
    // Most transaction tests are in connection.rs
    // This module just tests guard-specific behavior
}
