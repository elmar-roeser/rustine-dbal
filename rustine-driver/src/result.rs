//! Driver result trait

use rustine_core::{Result, SqlValue};

/// A result set from a query
pub trait DriverResult: Send + Sync {
    /// Get the next row from the result set
    fn next_row(&mut self) -> Result<Option<Vec<SqlValue>>>;

    /// Get all remaining rows
    fn all_rows(&mut self) -> Result<Vec<Vec<SqlValue>>> {
        let mut rows = Vec::new();
        while let Some(row) = self.next_row()? {
            rows.push(row);
        }
        Ok(rows)
    }

    /// Get the number of columns
    fn column_count(&self) -> usize;

    /// Get column names
    fn column_names(&self) -> &[String];

    /// Get the number of rows affected (for INSERT/UPDATE/DELETE)
    fn rows_affected(&self) -> u64;
}
