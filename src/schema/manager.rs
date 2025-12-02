//! Schema Manager for database introspection and manipulation

use crate::core::{Result, SqlValue};
use crate::driver::{DriverConnection, DriverResult};
use crate::platform::{ForeignKeyAction, Index, Platform, Table};

/// Schema Manager for introspecting and manipulating database schemas
///
/// The `SchemaManager` provides methods to:
/// - List tables, columns, indexes, and foreign keys
/// - Create and drop tables
/// - Create and drop indexes
#[derive(Debug)]
pub struct SchemaManager<'a, C: DriverConnection, P: Platform> {
    /// Database connection for executing schema queries
    connection: &'a C,
    /// Platform for generating SQL
    platform: &'a P,
}

impl<'a, C: DriverConnection, P: Platform> SchemaManager<'a, C, P> {
    /// Create a new `SchemaManager`
    #[must_use]
    pub const fn new(connection: &'a C, platform: &'a P) -> Self {
        Self {
            connection,
            platform,
        }
    }

    /// List all table names in the database
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn list_table_names(&self) -> Result<Vec<String>> {
        let sql = self.platform.get_list_tables_sql();
        let mut result = self.connection.query(sql).await?;
        let rows = result.all_rows()?;

        let mut tables = Vec::new();
        for row in rows {
            if let Some(SqlValue::String(name)) = row.first() {
                tables.push(name.clone());
            }
        }

        Ok(tables)
    }

    /// List all columns of a table
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn list_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let sql = self.platform.get_list_columns_sql(table_name);
        let mut result = self.connection.query(&sql).await?;
        let rows = result.all_rows()?;

        let mut columns = Vec::new();
        for row in rows {
            if let Some(info) = self.parse_column_row(&row) {
                columns.push(info);
            }
        }

        Ok(columns)
    }

    /// List all indexes of a table
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn list_table_indexes(&self, table_name: &str) -> Result<Vec<IndexInfo>> {
        let sql = self.platform.get_list_indexes_sql(table_name);
        let mut result = self.connection.query(&sql).await?;
        let rows = result.all_rows()?;

        let mut indexes = Vec::new();
        for row in rows {
            if let Some(info) = self.parse_index_row(&row) {
                indexes.push(info);
            }
        }

        Ok(indexes)
    }

    /// List all foreign keys of a table
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn list_table_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKeyInfo>> {
        let sql = self.platform.get_list_foreign_keys_sql(table_name);
        let mut result = self.connection.query(&sql).await?;
        let rows = result.all_rows()?;

        let mut fks = Vec::new();
        for row in rows {
            if let Some(info) = self.parse_foreign_key_row(&row) {
                fks.push(info);
            }
        }

        Ok(fks)
    }

    /// Check if a table exists
    ///
    /// # Errors
    ///
    /// Returns an error if listing tables fails.
    pub async fn table_exists(&self, table_name: &str) -> Result<bool> {
        let tables = self.list_table_names().await?;
        Ok(tables.iter().any(|t| t.eq_ignore_ascii_case(table_name)))
    }

    /// Get full table information including columns, indexes, and foreign keys
    ///
    /// # Errors
    ///
    /// Returns an error if any introspection query fails.
    pub async fn introspect_table(&self, table_name: &str) -> Result<TableInfo> {
        let columns = self.list_table_columns(table_name).await?;
        let indexes = self.list_table_indexes(table_name).await?;
        let foreign_keys = self.list_table_foreign_keys(table_name).await?;

        Ok(TableInfo {
            name: table_name.to_string(),
            columns,
            indexes,
            foreign_keys,
        })
    }

    /// Create a table from a Table definition
    ///
    /// # Errors
    ///
    /// Returns an error if the CREATE TABLE statement fails.
    pub async fn create_table(&self, table: &Table) -> Result<()> {
        let sql = self.platform.get_create_table_sql(table);
        self.connection.execute(&sql).await?;
        Ok(())
    }

    /// Drop a table
    ///
    /// # Errors
    ///
    /// Returns an error if the DROP TABLE statement fails.
    pub async fn drop_table(&self, table_name: &str) -> Result<()> {
        let sql = self.platform.get_drop_table_sql(table_name);
        self.connection.execute(&sql).await?;
        Ok(())
    }

    /// Drop a table if it exists
    ///
    /// # Errors
    ///
    /// Returns an error if the DROP TABLE IF EXISTS statement fails.
    pub async fn drop_table_if_exists(&self, table_name: &str) -> Result<()> {
        let sql = self.platform.get_drop_table_if_exists_sql(table_name);
        self.connection.execute(&sql).await?;
        Ok(())
    }

    /// Create an index
    ///
    /// # Errors
    ///
    /// Returns an error if the CREATE INDEX statement fails.
    pub async fn create_index(&self, table_name: &str, index: &Index) -> Result<()> {
        let sql = self.platform.get_create_index_sql(table_name, index);
        self.connection.execute(&sql).await?;
        Ok(())
    }

    /// Drop an index
    ///
    /// # Errors
    ///
    /// Returns an error if the DROP INDEX statement fails.
    pub async fn drop_index(&self, index_name: &str, table_name: &str) -> Result<()> {
        let sql = self.platform.get_drop_index_sql(index_name, table_name);
        self.connection.execute(&sql).await?;
        Ok(())
    }

    // ========================================================================
    // Platform-specific row parsing
    // ========================================================================

    /// Parse a column metadata row from the database
    fn parse_column_row(&self, row: &[SqlValue]) -> Option<ColumnInfo> {
        // The row format depends on the platform, but we try to handle common cases
        // PostgreSQL/MySQL: column_name, data_type, is_nullable, column_default, ...
        // SQLite (PRAGMA): cid, name, type, notnull, dflt_value, pk

        if row.is_empty() {
            return None;
        }

        let platform_name = self.platform.name();

        match platform_name {
            "sqlite" => self.parse_sqlite_column_row(row),
            _ => self.parse_standard_column_row(row),
        }
    }

    /// Parse a `SQLite` `PRAGMA` `table_info` row
    #[allow(clippy::unused_self)]
    fn parse_sqlite_column_row(&self, row: &[SqlValue]) -> Option<ColumnInfo> {
        // SQLite PRAGMA table_info returns: cid, name, type, notnull, dflt_value, pk
        if row.len() < 6 {
            return None;
        }

        let name = match &row[1] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let type_name = match &row[2] {
            SqlValue::String(s) => s.clone(),
            _ => String::new(),
        };

        let not_null = match &row[3] {
            SqlValue::I64(v) => *v != 0,
            SqlValue::I32(v) => *v != 0,
            SqlValue::Bool(v) => *v,
            _ => false,
        };

        let default = match &row[4] {
            SqlValue::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };

        let is_primary_key = match &row[5] {
            SqlValue::I64(v) => *v != 0,
            SqlValue::I32(v) => *v != 0,
            SqlValue::Bool(v) => *v,
            _ => false,
        };

        let is_auto_increment = is_primary_key && type_name.to_uppercase() == "INTEGER";

        // In SQLite, PRIMARY KEY columns are implicitly NOT NULL
        let nullable = if is_primary_key { false } else { !not_null };

        Some(ColumnInfo {
            name,
            type_name,
            nullable,
            default,
            is_primary_key,
            is_auto_increment,
        })
    }

    /// Parse a standard `information_schema` column row
    #[allow(clippy::unused_self)]
    fn parse_standard_column_row(&self, row: &[SqlValue]) -> Option<ColumnInfo> {
        // Standard information_schema format: column_name, data_type, is_nullable, column_default, ...
        if row.is_empty() {
            return None;
        }

        let name = match &row[0] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let type_name = if row.len() > 1 {
            match &row[1] {
                SqlValue::String(s) => s.clone(),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        let nullable = if row.len() > 2 {
            match &row[2] {
                SqlValue::String(s) => s.eq_ignore_ascii_case("YES"),
                SqlValue::Bool(b) => *b,
                _ => true,
            }
        } else {
            true
        };

        let default = if row.len() > 3 {
            match &row[3] {
                SqlValue::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };

        Some(ColumnInfo {
            name,
            type_name,
            nullable,
            default,
            is_primary_key: false, // Would need additional query
            is_auto_increment: false, // Would need additional query
        })
    }

    /// Parse an index metadata row from the database
    fn parse_index_row(&self, row: &[SqlValue]) -> Option<IndexInfo> {
        if row.is_empty() {
            return None;
        }

        let platform_name = self.platform.name();

        match platform_name {
            "sqlite" => self.parse_sqlite_index_row(row),
            _ => self.parse_standard_index_row(row),
        }
    }

    /// Parse a `SQLite` `PRAGMA` `index_list` row
    #[allow(clippy::unused_self)]
    fn parse_sqlite_index_row(&self, row: &[SqlValue]) -> Option<IndexInfo> {
        // SQLite PRAGMA index_list returns: seq, name, unique, origin, partial
        if row.len() < 3 {
            return None;
        }

        let name = match &row[1] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let unique = match &row[2] {
            SqlValue::I64(v) => *v != 0,
            SqlValue::I32(v) => *v != 0,
            SqlValue::Bool(v) => *v,
            _ => false,
        };

        let origin = if row.len() > 3 {
            match &row[3] {
                SqlValue::String(s) => s.clone(),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        Some(IndexInfo {
            name,
            columns: Vec::new(), // Would need PRAGMA index_info to get columns
            unique,
            primary: origin == "pk",
        })
    }

    /// Parse a standard `information_schema` index row
    #[allow(clippy::unused_self)]
    fn parse_standard_index_row(&self, row: &[SqlValue]) -> Option<IndexInfo> {
        if row.is_empty() {
            return None;
        }

        let name = match &row[0] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let column = if row.len() > 1 {
            match &row[1] {
                SqlValue::String(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };

        let unique = if row.len() > 2 {
            match &row[2] {
                SqlValue::Bool(b) => *b,
                SqlValue::I64(v) => *v != 0,
                SqlValue::I32(v) => *v != 0,
                _ => false,
            }
        } else {
            false
        };

        let primary = if row.len() > 3 {
            match &row[3] {
                SqlValue::Bool(b) => *b,
                SqlValue::I64(v) => *v != 0,
                SqlValue::I32(v) => *v != 0,
                _ => false,
            }
        } else {
            false
        };

        Some(IndexInfo {
            name,
            columns: column.into_iter().collect(),
            unique,
            primary,
        })
    }

    /// Parse a foreign key metadata row from the database
    fn parse_foreign_key_row(&self, row: &[SqlValue]) -> Option<ForeignKeyInfo> {
        if row.is_empty() {
            return None;
        }

        let platform_name = self.platform.name();

        match platform_name {
            "sqlite" => self.parse_sqlite_foreign_key_row(row),
            _ => self.parse_standard_foreign_key_row(row),
        }
    }

    /// Parse a `SQLite` `PRAGMA` `foreign_key_list` row
    fn parse_sqlite_foreign_key_row(&self, row: &[SqlValue]) -> Option<ForeignKeyInfo> {
        // SQLite PRAGMA foreign_key_list returns: id, seq, table, from, to, on_update, on_delete, match
        if row.len() < 5 {
            return None;
        }

        let foreign_table = match &row[2] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let local_column = match &row[3] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let foreign_column = match &row[4] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let on_update = if row.len() > 5 {
            self.parse_fk_action(&row[5])
        } else {
            ForeignKeyAction::NoAction
        };

        let on_delete = if row.len() > 6 {
            self.parse_fk_action(&row[6])
        } else {
            ForeignKeyAction::NoAction
        };

        Some(ForeignKeyInfo {
            name: String::new(), // SQLite doesn't name FK constraints
            local_columns: vec![local_column],
            foreign_table,
            foreign_columns: vec![foreign_column],
            on_update,
            on_delete,
        })
    }

    /// Parse a standard `information_schema` foreign key row
    #[allow(clippy::unused_self)]
    fn parse_standard_foreign_key_row(&self, row: &[SqlValue]) -> Option<ForeignKeyInfo> {
        // Standard: constraint_name, column_name, foreign_table_name, foreign_column_name
        if row.len() < 4 {
            return None;
        }

        let name = match &row[0] {
            SqlValue::String(s) => s.clone(),
            _ => String::new(),
        };

        let local_column = match &row[1] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let foreign_table = match &row[2] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        let foreign_column = match &row[3] {
            SqlValue::String(s) => s.clone(),
            _ => return None,
        };

        Some(ForeignKeyInfo {
            name,
            local_columns: vec![local_column],
            foreign_table,
            foreign_columns: vec![foreign_column],
            on_update: ForeignKeyAction::NoAction,
            on_delete: ForeignKeyAction::NoAction,
        })
    }

    /// Parse a foreign key action from a SQL value
    #[allow(clippy::unused_self)]
    fn parse_fk_action(&self, value: &SqlValue) -> ForeignKeyAction {
        match value {
            SqlValue::String(s) => match s.to_uppercase().as_str() {
                "CASCADE" => ForeignKeyAction::Cascade,
                "SET NULL" => ForeignKeyAction::SetNull,
                "SET DEFAULT" => ForeignKeyAction::SetDefault,
                "RESTRICT" => ForeignKeyAction::Restrict,
                _ => ForeignKeyAction::NoAction,
            },
            _ => ForeignKeyAction::NoAction,
        }
    }
}

/// Information about a database column
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// SQL type name as reported by the database
    pub type_name: String,
    /// Whether the column allows NULL values
    pub nullable: bool,
    /// Default value expression
    pub default: Option<String>,
    /// Whether this column is part of the primary key
    pub is_primary_key: bool,
    /// Whether this column auto-increments
    pub is_auto_increment: bool,
}

/// Information about a database index
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// Index name
    pub name: String,
    /// Columns in the index
    pub columns: Vec<String>,
    /// Whether this is a unique index
    pub unique: bool,
    /// Whether this is the primary key
    pub primary: bool,
}

/// Information about a foreign key constraint
#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    /// Constraint name
    pub name: String,
    /// Local column names
    pub local_columns: Vec<String>,
    /// Referenced table name
    pub foreign_table: String,
    /// Referenced column names
    pub foreign_columns: Vec<String>,
    /// ON UPDATE action
    pub on_update: ForeignKeyAction,
    /// ON DELETE action
    pub on_delete: ForeignKeyAction,
}

/// Complete table information from introspection
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Table name
    pub name: String,
    /// Columns
    pub columns: Vec<ColumnInfo>,
    /// Indexes
    pub indexes: Vec<IndexInfo>,
    /// Foreign keys
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

impl TableInfo {
    /// Get the primary key columns
    #[must_use]
    pub fn primary_key_columns(&self) -> Vec<&str> {
        self.columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.name.as_str())
            .collect()
    }

    /// Check if the table has a specific column
    #[must_use]
    pub fn has_column(&self, name: &str) -> bool {
        self.columns.iter().any(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// Get column by name
    #[must_use]
    pub fn get_column(&self, name: &str) -> Option<&ColumnInfo> {
        self.columns.iter().find(|c| c.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_info() {
        let info = ColumnInfo {
            name: "id".to_string(),
            type_name: "INTEGER".to_string(),
            nullable: false,
            default: None,
            is_primary_key: true,
            is_auto_increment: true,
        };

        assert_eq!(info.name, "id");
        assert!(!info.nullable);
        assert!(info.is_primary_key);
    }

    #[test]
    fn test_table_info_primary_key() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    type_name: "INTEGER".to_string(),
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    is_auto_increment: true,
                },
                ColumnInfo {
                    name: "name".to_string(),
                    type_name: "TEXT".to_string(),
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    is_auto_increment: false,
                },
            ],
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
        };

        assert_eq!(info.primary_key_columns(), vec!["id"]);
        assert!(info.has_column("id"));
        assert!(info.has_column("name"));
        assert!(!info.has_column("email"));
    }

    #[test]
    fn test_table_info_get_column() {
        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnInfo {
                name: "MyColumn".to_string(),
                type_name: "TEXT".to_string(),
                nullable: true,
                default: None,
                is_primary_key: false,
                is_auto_increment: false,
            }],
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
        };

        // Case-insensitive lookup
        assert!(info.get_column("mycolumn").is_some());
        assert!(info.get_column("MYCOLUMN").is_some());
        assert!(info.get_column("nonexistent").is_none());
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod sqlite_tests {
    use super::*;
    use crate::driver::{Driver, SqliteDriver};
    use crate::platform::{Column, SqlType, SqlitePlatform};

    async fn setup_connection() -> <SqliteDriver as Driver>::Connection {
        let driver = SqliteDriver::new();
        let params = crate::core::ConnectionParams::sqlite_memory();
        driver.connect(&params).await.unwrap()
    }

    #[tokio::test]
    async fn test_list_table_names_empty() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        let tables = manager.list_table_names().await.unwrap();
        assert!(tables.is_empty());
    }

    #[tokio::test]
    async fn test_create_and_list_tables() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Create a table
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Integer).not_null().auto_increment())
            .column(Column::new("name", SqlType::Text).not_null())
            .column(Column::new("email", SqlType::Text));

        manager.create_table(&table).await.unwrap();

        // List tables
        let tables = manager.list_table_names().await.unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0], "users");
    }

    #[tokio::test]
    async fn test_table_exists() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        assert!(!manager.table_exists("users").await.unwrap());

        // Create table
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Integer).not_null());

        manager.create_table(&table).await.unwrap();

        assert!(manager.table_exists("users").await.unwrap());
        assert!(!manager.table_exists("posts").await.unwrap());
    }

    #[tokio::test]
    async fn test_drop_table() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Create table
        let table = Table::new("test_table")
            .column(Column::new("id", SqlType::Integer).not_null());

        manager.create_table(&table).await.unwrap();
        assert!(manager.table_exists("test_table").await.unwrap());

        // Drop table
        manager.drop_table("test_table").await.unwrap();
        assert!(!manager.table_exists("test_table").await.unwrap());
    }

    #[tokio::test]
    async fn test_drop_table_if_exists() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Should not error even if table doesn't exist
        manager.drop_table_if_exists("nonexistent").await.unwrap();

        // Create and drop
        let table = Table::new("test")
            .column(Column::new("id", SqlType::Integer));
        manager.create_table(&table).await.unwrap();
        manager.drop_table_if_exists("test").await.unwrap();

        assert!(!manager.table_exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_table_columns() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Create table with various column types
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Integer).not_null().auto_increment())
            .column(Column::new("name", SqlType::Text).not_null())
            .column(Column::new("email", SqlType::Text))
            .column(Column::new("age", SqlType::Integer).default("0"));

        manager.create_table(&table).await.unwrap();

        // List columns
        let columns = manager.list_table_columns("users").await.unwrap();
        assert_eq!(columns.len(), 4);

        // Check id column
        let id_col = columns.iter().find(|c| c.name == "id").unwrap();
        assert_eq!(id_col.type_name, "INTEGER");
        assert!(!id_col.nullable);
        assert!(id_col.is_primary_key);

        // Check name column
        let name_col = columns.iter().find(|c| c.name == "name").unwrap();
        assert_eq!(name_col.type_name, "TEXT");
        assert!(!name_col.nullable);

        // Check email column (nullable)
        let email_col = columns.iter().find(|c| c.name == "email").unwrap();
        assert!(email_col.nullable);

        // Check age column (with default)
        let age_col = columns.iter().find(|c| c.name == "age").unwrap();
        assert!(age_col.default.is_some());
    }

    #[tokio::test]
    async fn test_introspect_table() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Create table
        let table = Table::new("products")
            .column(Column::new("id", SqlType::Integer).not_null().auto_increment())
            .column(Column::new("name", SqlType::Text).not_null())
            .column(Column::new("price", SqlType::Float));

        manager.create_table(&table).await.unwrap();

        // Introspect
        let info = manager.introspect_table("products").await.unwrap();

        assert_eq!(info.name, "products");
        assert_eq!(info.columns.len(), 3);
        assert!(info.has_column("id"));
        assert!(info.has_column("name"));
        assert!(info.has_column("price"));
        assert!(!info.has_column("description"));

        // Check primary key
        let pk_cols = info.primary_key_columns();
        assert_eq!(pk_cols, vec!["id"]);
    }

    #[tokio::test]
    async fn test_list_indexes() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Create table
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Integer).not_null().auto_increment())
            .column(Column::new("email", SqlType::Text).not_null());

        manager.create_table(&table).await.unwrap();

        // Create an index
        let index = crate::platform::Index::new("idx_users_email", vec!["email".to_string()]);
        manager.create_index("users", &index).await.unwrap();

        // List indexes
        let indexes = manager.list_table_indexes("users").await.unwrap();

        // Should have at least the index we created
        let email_idx = indexes.iter().find(|i| i.name == "idx_users_email");
        assert!(email_idx.is_some());
    }

    #[tokio::test]
    async fn test_create_unique_index() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Create table
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Integer).not_null())
            .column(Column::new("email", SqlType::Text).not_null());

        manager.create_table(&table).await.unwrap();

        // Create unique index
        let index = crate::platform::Index::unique("idx_users_email_unique", vec!["email".to_string()]);
        manager.create_index("users", &index).await.unwrap();

        // List indexes
        let indexes = manager.list_table_indexes("users").await.unwrap();
        let email_idx = indexes.iter().find(|i| i.name == "idx_users_email_unique");
        assert!(email_idx.is_some());
        assert!(email_idx.unwrap().unique);
    }

    #[tokio::test]
    async fn test_foreign_keys() {
        let conn = setup_connection().await;
        let platform = SqlitePlatform;
        let manager = SchemaManager::new(&conn, &platform);

        // Enable foreign keys in SQLite
        conn.execute("PRAGMA foreign_keys = ON").await.unwrap();

        // Create parent table
        let users = Table::new("users")
            .column(Column::new("id", SqlType::Integer).not_null().auto_increment());
        manager.create_table(&users).await.unwrap();

        // Create child table with foreign key
        let posts = Table::new("posts")
            .column(Column::new("id", SqlType::Integer).not_null().auto_increment())
            .column(Column::new("user_id", SqlType::Integer).not_null())
            .foreign_key(crate::platform::ForeignKey {
                name: "fk_posts_user".to_string(),
                local_columns: vec!["user_id".to_string()],
                foreign_table: "users".to_string(),
                foreign_columns: vec!["id".to_string()],
                on_delete: ForeignKeyAction::Cascade,
                on_update: ForeignKeyAction::NoAction,
            });

        manager.create_table(&posts).await.unwrap();

        // List foreign keys
        let fks = manager.list_table_foreign_keys("posts").await.unwrap();
        assert_eq!(fks.len(), 1);

        let fk = &fks[0];
        assert_eq!(fk.foreign_table, "users");
        assert_eq!(fk.local_columns, vec!["user_id"]);
        assert_eq!(fk.foreign_columns, vec!["id"]);
    }
}
