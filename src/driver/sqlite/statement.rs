//! SQLite prepared statement implementation

use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

use crate::core::{QueryError, Result, SqlValue};
use crate::driver::DriverStatement;

use super::SqliteResult;

/// SQLite prepared statement
pub struct SqliteStatement {
    pool: SqlitePool,
    sql: String,
    positional_params: HashMap<usize, SqlValue>,
    named_params: HashMap<String, SqlValue>,
}

impl SqliteStatement {
    /// Create a new prepared statement
    pub(crate) fn new(pool: SqlitePool, sql: String) -> Self {
        Self {
            pool,
            sql,
            positional_params: HashMap::new(),
            named_params: HashMap::new(),
        }
    }

    /// Build the final SQL with bound parameters
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
        // First collect all positional params in order
        let mut positions: Vec<_> = self.positional_params.keys().copied().collect();
        positions.sort();

        for pos in positions {
            if let Some(value) = self.positional_params.get(&pos) {
                values.push(value.clone());
            }
        }

        (sql, values)
    }

    /// Execute a query with the given SQL and values
    async fn execute_with_values(&self, sql: &str, values: &[SqlValue]) -> Result<SqliteResult> {
        let mut query = sqlx::query(sql);

        // Bind all values
        for value in values {
            query = match value {
                SqlValue::Null => query.bind(None::<String>),
                SqlValue::Bool(v) => query.bind(*v),
                SqlValue::I8(v) => query.bind(*v as i32),
                SqlValue::I16(v) => query.bind(*v as i32),
                SqlValue::I32(v) => query.bind(*v),
                SqlValue::I64(v) => query.bind(*v),
                SqlValue::U32(v) => query.bind(*v as i64),
                SqlValue::U64(v) => query.bind(*v as i64),
                SqlValue::F32(v) => query.bind(*v as f64),
                SqlValue::F64(v) => query.bind(*v),
                SqlValue::String(v) => query.bind(v.as_str()),
                SqlValue::Bytes(v) => query.bind(v.as_slice()),
                #[cfg(feature = "chrono")]
                SqlValue::Date(v) => query.bind(v.to_string()),
                #[cfg(feature = "chrono")]
                SqlValue::Time(v) => query.bind(v.to_string()),
                #[cfg(feature = "chrono")]
                SqlValue::DateTime(v) => query.bind(v.to_string()),
                #[cfg(feature = "chrono")]
                SqlValue::DateTimeUtc(v) => query.bind(v.to_rfc3339()),
                #[cfg(feature = "uuid")]
                SqlValue::Uuid(v) => query.bind(v.to_string()),
                #[cfg(feature = "json")]
                SqlValue::Json(v) => query.bind(v.to_string()),
                #[cfg(feature = "decimal")]
                SqlValue::Decimal(v) => query.bind(v.to_string()),
            };
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| QueryError::ExecutionFailed {
                message: e.to_string(),
                sql: Some(sql.to_string()),
            })?;

        if rows.is_empty() {
            return Ok(SqliteResult::new(Vec::new(), Vec::new(), 0));
        }

        // Extract column names
        use sqlx::Column;
        let column_names: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        // Convert rows to SqlValue
        let data: Vec<Vec<SqlValue>> = rows
            .iter()
            .map(|row| {
                let columns = row.columns();
                let mut values = Vec::with_capacity(columns.len());

                for (i, col) in columns.iter().enumerate() {
                    let type_name = col.type_info().to_string().to_uppercase();
                    let value = match type_name.as_str() {
                        "INTEGER" | "INT" | "BIGINT" => {
                            row.try_get::<i64, _>(i)
                                .map(SqlValue::I64)
                                .unwrap_or(SqlValue::Null)
                        }
                        "REAL" | "DOUBLE" | "FLOAT" => {
                            row.try_get::<f64, _>(i)
                                .map(SqlValue::F64)
                                .unwrap_or(SqlValue::Null)
                        }
                        "TEXT" | "VARCHAR" | "CHAR" => {
                            row.try_get::<String, _>(i)
                                .map(SqlValue::String)
                                .unwrap_or(SqlValue::Null)
                        }
                        "BLOB" => {
                            row.try_get::<Vec<u8>, _>(i)
                                .map(SqlValue::Bytes)
                                .unwrap_or(SqlValue::Null)
                        }
                        "BOOLEAN" | "BOOL" => {
                            row.try_get::<bool, _>(i)
                                .map(SqlValue::Bool)
                                .unwrap_or(SqlValue::Null)
                        }
                        _ => {
                            row.try_get::<String, _>(i)
                                .map(SqlValue::String)
                                .unwrap_or(SqlValue::Null)
                        }
                    };
                    values.push(value);
                }

                values
            })
            .collect();

        Ok(SqliteResult::new(data, column_names, 0))
    }

    /// Execute an update with the given SQL and values
    async fn execute_update_with_values(&self, sql: &str, values: &[SqlValue]) -> Result<u64> {
        let mut query = sqlx::query(sql);

        // Bind all values
        for value in values {
            query = match value {
                SqlValue::Null => query.bind(None::<String>),
                SqlValue::Bool(v) => query.bind(*v),
                SqlValue::I8(v) => query.bind(*v as i32),
                SqlValue::I16(v) => query.bind(*v as i32),
                SqlValue::I32(v) => query.bind(*v),
                SqlValue::I64(v) => query.bind(*v),
                SqlValue::U32(v) => query.bind(*v as i64),
                SqlValue::U64(v) => query.bind(*v as i64),
                SqlValue::F32(v) => query.bind(*v as f64),
                SqlValue::F64(v) => query.bind(*v),
                SqlValue::String(v) => query.bind(v.as_str()),
                SqlValue::Bytes(v) => query.bind(v.as_slice()),
                #[cfg(feature = "chrono")]
                SqlValue::Date(v) => query.bind(v.to_string()),
                #[cfg(feature = "chrono")]
                SqlValue::Time(v) => query.bind(v.to_string()),
                #[cfg(feature = "chrono")]
                SqlValue::DateTime(v) => query.bind(v.to_string()),
                #[cfg(feature = "chrono")]
                SqlValue::DateTimeUtc(v) => query.bind(v.to_rfc3339()),
                #[cfg(feature = "uuid")]
                SqlValue::Uuid(v) => query.bind(v.to_string()),
                #[cfg(feature = "json")]
                SqlValue::Json(v) => query.bind(v.to_string()),
                #[cfg(feature = "decimal")]
                SqlValue::Decimal(v) => query.bind(v.to_string()),
            };
        }

        let result = query
            .execute(&self.pool)
            .await
            .map_err(|e| QueryError::ExecutionFailed {
                message: e.to_string(),
                sql: Some(sql.to_string()),
            })?;

        Ok(result.rows_affected())
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
        let (sql, values) = self.build_query();
        self.execute_with_values(&sql, &values).await
    }

    async fn execute_update(&self) -> Result<u64> {
        let (sql, values) = self.build_query();
        self.execute_update_with_values(&sql, &values).await
    }

    fn sql(&self) -> &str {
        &self.sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{Driver, DriverConnection, DriverResult};
    use super::super::SqliteDriver;

    #[tokio::test]
    async fn test_prepared_statement_positional() {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        let conn = driver.connect(&params).await.unwrap();

        // Create table
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();

        // Prepare and execute insert
        let mut stmt = conn.prepare("INSERT INTO test (id, name) VALUES (?, ?)").await.unwrap();
        stmt.bind(0, SqlValue::I64(1)).unwrap();
        stmt.bind(1, SqlValue::String("Alice".to_string())).unwrap();
        let affected = stmt.execute_update().await.unwrap();

        assert_eq!(affected, 1);

        // Verify
        let mut result = conn.query("SELECT * FROM test").await.unwrap();
        let rows = result.all_rows().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], SqlValue::I64(1));
    }

    #[tokio::test]
    async fn test_prepared_statement_named() {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        let conn = driver.connect(&params).await.unwrap();

        // Create table
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();

        // Prepare and execute insert with named parameters
        let mut stmt = conn.prepare("INSERT INTO test (id, name) VALUES (:id, :name)").await.unwrap();
        stmt.bind_named("id", SqlValue::I64(1)).unwrap();
        stmt.bind_named("name", SqlValue::String("Bob".to_string())).unwrap();
        let affected = stmt.execute_update().await.unwrap();

        assert_eq!(affected, 1);
    }

    #[tokio::test]
    async fn test_sql_getter() {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        let conn = driver.connect(&params).await.unwrap();

        let stmt = conn.prepare("SELECT 1").await.unwrap();
        assert_eq!(stmt.sql(), "SELECT 1");
    }
}
