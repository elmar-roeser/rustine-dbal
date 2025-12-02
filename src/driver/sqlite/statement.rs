//! SQLite prepared statement implementation

use async_trait::async_trait;
use sqlx::sqlite::SqliteConnection as SqlxSqliteConnection;
use std::collections::HashMap;
use tokio::sync::MutexGuard;

use crate::core::{Result, SqlValue};
use crate::driver::DriverStatement;

use super::SqliteResult;

/// SQLite prepared statement
///
/// Note: This implementation stores the SQL and parameters, and executes
/// them when `execute()` or `execute_update()` is called. For true prepared
/// statement support with connection binding, use the Connection directly.
pub struct SqliteStatement {
    sql: String,
    positional_params: HashMap<usize, SqlValue>,
    named_params: HashMap<String, SqlValue>,
}

impl SqliteStatement {
    /// Create a new prepared statement (simplified version without connection)
    #[allow(dead_code)]
    pub(crate) fn new(sql: String) -> Self {
        Self {
            sql,
            positional_params: HashMap::new(),
            named_params: HashMap::new(),
        }
    }

    /// Create a new prepared statement with connection reference
    /// Note: The connection guard is immediately dropped as we store only the SQL
    pub(crate) fn new_with_connection<'a>(
        sql: String,
        _conn: MutexGuard<'a, SqlxSqliteConnection>,
    ) -> Self {
        Self {
            sql,
            positional_params: HashMap::new(),
            named_params: HashMap::new(),
        }
    }

    /// Build the final SQL with bound parameters
    #[allow(dead_code)]
    fn build_query(&self) -> (String, Vec<SqlValue>) {
        let mut sql = self.sql.clone();
        let mut values = Vec::new();

        // Handle named parameters (convert :name to ?N format)
        for (name, value) in &self.named_params {
            let placeholder = format!(":{}", name);
            if sql.contains(&placeholder) {
                values.push(value.clone());
                sql = sql.replace(&placeholder, &format!("?{}", values.len()));
            }
        }

        // Handle positional parameters
        let mut positions: Vec<_> = self.positional_params.keys().copied().collect();
        positions.sort();

        for pos in positions {
            if let Some(value) = self.positional_params.get(&pos) {
                values.push(value.clone());
            }
        }

        (sql, values)
    }
}

#[async_trait]
impl DriverStatement for SqliteStatement {
    type Result = SqliteResult;

    fn bind(&mut self, position: usize, value: SqlValue) -> Result<()> {
        self.positional_params.insert(position, value);
        Ok(())
    }

    fn bind_named(&mut self, name: &str, value: SqlValue) -> Result<()> {
        self.named_params.insert(name.to_string(), value);
        Ok(())
    }

    async fn execute(&self) -> Result<Self::Result> {
        // This simplified implementation cannot execute without a connection
        // In practice, users should use Connection::query() with parameters
        Err(crate::core::Error::driver_message(
            "SqliteStatement::execute() requires a connection. Use Connection::query() instead.",
        ))
    }

    async fn execute_update(&self) -> Result<u64> {
        // This simplified implementation cannot execute without a connection
        // In practice, users should use Connection::execute() with parameters
        Err(crate::core::Error::driver_message(
            "SqliteStatement::execute_update() requires a connection. Use Connection::execute() instead.",
        ))
    }

    fn sql(&self) -> &str {
        &self.sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_positional() {
        let mut stmt = SqliteStatement::new("INSERT INTO test VALUES (?, ?)".to_string());
        stmt.bind(0, SqlValue::I64(1)).unwrap();
        stmt.bind(1, SqlValue::String("Alice".to_string())).unwrap();

        let (sql, values) = stmt.build_query();
        assert_eq!(sql, "INSERT INTO test VALUES (?, ?)");
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], SqlValue::I64(1));
    }

    #[test]
    fn test_bind_named() {
        let mut stmt =
            SqliteStatement::new("INSERT INTO test VALUES (:id, :name)".to_string());
        stmt.bind_named("id", SqlValue::I64(1)).unwrap();
        stmt.bind_named("name", SqlValue::String("Bob".to_string()))
            .unwrap();

        let (_, values) = stmt.build_query();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_sql_getter() {
        let stmt = SqliteStatement::new("SELECT 1".to_string());
        assert_eq!(stmt.sql(), "SELECT 1");
    }
}
