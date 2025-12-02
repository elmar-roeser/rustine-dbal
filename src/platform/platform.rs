//! Platform trait for SQL dialect abstraction

use super::types::{Column, Index, SqlType, Table};

/// A database platform that generates platform-specific SQL
pub trait Platform: Send + Sync {
    /// Get the name of this platform
    fn name(&self) -> &'static str;

    /// Get the identifier quote character
    fn quote_identifier_char(&self) -> char;

    /// Quote an identifier (table name, column name, etc.)
    fn quote_identifier(&self, identifier: &str) -> String {
        let quote = self.quote_identifier_char();
        format!("{}{}{}", quote, identifier.replace(quote, &format!("{}{}", quote, quote)), quote)
    }

    /// Quote a string literal
    fn quote_string(&self, value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }

    /// Get the SQL for LIMIT/OFFSET
    fn limit_offset_sql(&self, limit: Option<u64>, offset: Option<u64>) -> String {
        let mut sql = String::new();
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        sql
    }

    /// Get the datetime format string
    fn datetime_format(&self) -> &'static str {
        "%Y-%m-%d %H:%M:%S"
    }

    /// Get the date format string
    fn date_format(&self) -> &'static str {
        "%Y-%m-%d"
    }

    /// Get the time format string
    fn time_format(&self) -> &'static str {
        "%H:%M:%S"
    }

    /// Check if this platform supports savepoints
    fn supports_savepoints(&self) -> bool {
        true
    }

    /// Check if this platform supports RETURNING clause
    fn supports_returning(&self) -> bool {
        false
    }

    /// Get the SQL for creating a savepoint
    fn create_savepoint_sql(&self, name: &str) -> String {
        format!("SAVEPOINT {}", self.quote_identifier(name))
    }

    /// Get the SQL for releasing a savepoint
    fn release_savepoint_sql(&self, name: &str) -> String {
        format!("RELEASE SAVEPOINT {}", self.quote_identifier(name))
    }

    /// Get the SQL for rolling back to a savepoint
    fn rollback_savepoint_sql(&self, name: &str) -> String {
        format!("ROLLBACK TO SAVEPOINT {}", self.quote_identifier(name))
    }

    /// Get the parameter placeholder style
    fn parameter_placeholder(&self, index: usize) -> String;

    /// Get the current timestamp function
    fn current_timestamp_sql(&self) -> &'static str {
        "CURRENT_TIMESTAMP"
    }

    /// Get the current date function
    fn current_date_sql(&self) -> &'static str {
        "CURRENT_DATE"
    }

    /// Get the current time function
    fn current_time_sql(&self) -> &'static str {
        "CURRENT_TIME"
    }

    // ========================================================================
    // Type Mapping
    // ========================================================================

    /// Get the SQL type name for a given SqlType
    fn get_type_declaration(&self, sql_type: &SqlType) -> String;

    /// Get the SQL for a column definition
    fn get_column_declaration(&self, column: &Column) -> String {
        let mut sql = format!(
            "{} {}",
            self.quote_identifier(&column.name),
            self.get_type_declaration(&column.sql_type)
        );

        if !column.nullable {
            sql.push_str(" NOT NULL");
        }

        if let Some(ref default) = column.default {
            sql.push_str(" DEFAULT ");
            sql.push_str(default);
        }

        sql
    }

    // ========================================================================
    // DDL Generation
    // ========================================================================

    /// Generate CREATE TABLE SQL
    fn get_create_table_sql(&self, table: &Table) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.quote_identifier(&table.name));

        // Columns
        let column_defs: Vec<String> = table
            .columns
            .iter()
            .map(|col| format!("    {}", self.get_column_declaration(col)))
            .collect();
        sql.push_str(&column_defs.join(",\n"));

        // Primary key
        if let Some(pk_cols) = table.primary_key_columns() {
            let pk_col_names: Vec<String> = pk_cols
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            sql.push_str(&format!(",\n    PRIMARY KEY ({})", pk_col_names.join(", ")));
        }

        // Unique indexes as constraints
        for index in &table.indexes {
            if index.unique && !index.primary {
                let col_names: Vec<String> = index
                    .columns
                    .iter()
                    .map(|c| self.quote_identifier(c))
                    .collect();
                if !index.name.is_empty() {
                    sql.push_str(&format!(
                        ",\n    CONSTRAINT {} UNIQUE ({})",
                        self.quote_identifier(&index.name),
                        col_names.join(", ")
                    ));
                } else {
                    sql.push_str(&format!(",\n    UNIQUE ({})", col_names.join(", ")));
                }
            }
        }

        // Foreign keys
        for fk in &table.foreign_keys {
            let local_cols: Vec<String> = fk
                .local_columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            let foreign_cols: Vec<String> = fk
                .foreign_columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();

            sql.push_str(&format!(
                ",\n    CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})",
                self.quote_identifier(&fk.name),
                local_cols.join(", "),
                self.quote_identifier(&fk.foreign_table),
                foreign_cols.join(", ")
            ));

            if fk.on_delete != super::types::ForeignKeyAction::NoAction {
                sql.push_str(&format!(" ON DELETE {}", fk.on_delete.as_sql()));
            }
            if fk.on_update != super::types::ForeignKeyAction::NoAction {
                sql.push_str(&format!(" ON UPDATE {}", fk.on_update.as_sql()));
            }
        }

        sql.push_str("\n)");
        sql
    }

    /// Generate DROP TABLE SQL
    fn get_drop_table_sql(&self, table_name: &str) -> String {
        format!("DROP TABLE {}", self.quote_identifier(table_name))
    }

    /// Generate DROP TABLE IF EXISTS SQL
    fn get_drop_table_if_exists_sql(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS {}", self.quote_identifier(table_name))
    }

    /// Generate CREATE INDEX SQL
    fn get_create_index_sql(&self, table_name: &str, index: &Index) -> String {
        let col_names: Vec<String> = index
            .columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();

        let unique = if index.unique { "UNIQUE " } else { "" };

        format!(
            "CREATE {}INDEX {} ON {} ({})",
            unique,
            self.quote_identifier(&index.name),
            self.quote_identifier(table_name),
            col_names.join(", ")
        )
    }

    /// Generate DROP INDEX SQL
    fn get_drop_index_sql(&self, index_name: &str, _table_name: &str) -> String {
        format!("DROP INDEX {}", self.quote_identifier(index_name))
    }

    // ========================================================================
    // Schema Introspection SQL
    // ========================================================================

    /// Get SQL to list all tables in the database
    fn get_list_tables_sql(&self) -> &'static str;

    /// Get SQL to list columns of a table
    fn get_list_columns_sql(&self, table_name: &str) -> String;

    /// Get SQL to list indexes of a table
    fn get_list_indexes_sql(&self, table_name: &str) -> String;

    /// Get SQL to list foreign keys of a table
    fn get_list_foreign_keys_sql(&self, table_name: &str) -> String;
}

/// PostgreSQL platform
pub struct PostgresPlatform;

impl Platform for PostgresPlatform {
    fn name(&self) -> &'static str {
        "postgresql"
    }

    fn quote_identifier_char(&self) -> char {
        '"'
    }

    fn supports_returning(&self) -> bool {
        true
    }

    fn parameter_placeholder(&self, index: usize) -> String {
        format!("${}", index + 1)
    }

    fn get_type_declaration(&self, sql_type: &SqlType) -> String {
        match sql_type {
            SqlType::SmallInt => "SMALLINT".to_string(),
            SqlType::Integer => "INTEGER".to_string(),
            SqlType::BigInt => "BIGINT".to_string(),
            SqlType::Float => "REAL".to_string(),
            SqlType::Double => "DOUBLE PRECISION".to_string(),
            SqlType::Decimal { precision, scale } => format!("NUMERIC({}, {})", precision, scale),
            SqlType::Char { length } => format!("CHAR({})", length),
            SqlType::Varchar { length } => format!("VARCHAR({})", length),
            SqlType::Text => "TEXT".to_string(),
            SqlType::Binary { length: _ } => "BYTEA".to_string(), // PostgreSQL uses BYTEA
            SqlType::VarBinary { length: _ } => "BYTEA".to_string(),
            SqlType::Blob => "BYTEA".to_string(),
            SqlType::Boolean => "BOOLEAN".to_string(),
            SqlType::Date => "DATE".to_string(),
            SqlType::Time { precision } => match precision {
                Some(p) => format!("TIME({})", p),
                None => "TIME".to_string(),
            },
            SqlType::Timestamp { precision } => match precision {
                Some(p) => format!("TIMESTAMP({})", p),
                None => "TIMESTAMP".to_string(),
            },
            SqlType::TimestampTz { precision } => match precision {
                Some(p) => format!("TIMESTAMP({}) WITH TIME ZONE", p),
                None => "TIMESTAMP WITH TIME ZONE".to_string(),
            },
            SqlType::Uuid => "UUID".to_string(),
            SqlType::Json => "JSONB".to_string(),
            SqlType::Serial => "SERIAL".to_string(),
            SqlType::BigSerial => "BIGSERIAL".to_string(),
        }
    }

    fn get_list_tables_sql(&self) -> &'static str {
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
    }

    fn get_list_columns_sql(&self, table_name: &str) -> String {
        format!(
            "SELECT column_name, data_type, is_nullable, column_default, character_maximum_length, numeric_precision, numeric_scale \
             FROM information_schema.columns WHERE table_schema = 'public' AND table_name = '{}' ORDER BY ordinal_position",
            table_name
        )
    }

    fn get_list_indexes_sql(&self, table_name: &str) -> String {
        format!(
            "SELECT i.relname AS index_name, a.attname AS column_name, ix.indisunique AS is_unique, ix.indisprimary AS is_primary \
             FROM pg_class t, pg_class i, pg_index ix, pg_attribute a \
             WHERE t.oid = ix.indrelid AND i.oid = ix.indexrelid AND a.attrelid = t.oid AND a.attnum = ANY(ix.indkey) \
             AND t.relkind = 'r' AND t.relname = '{}'",
            table_name
        )
    }

    fn get_list_foreign_keys_sql(&self, table_name: &str) -> String {
        format!(
            "SELECT tc.constraint_name, kcu.column_name, ccu.table_name AS foreign_table_name, ccu.column_name AS foreign_column_name \
             FROM information_schema.table_constraints AS tc \
             JOIN information_schema.key_column_usage AS kcu ON tc.constraint_name = kcu.constraint_name \
             JOIN information_schema.constraint_column_usage AS ccu ON ccu.constraint_name = tc.constraint_name \
             WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_name = '{}'",
            table_name
        )
    }
}

/// MySQL platform
pub struct MySqlPlatform;

impl Platform for MySqlPlatform {
    fn name(&self) -> &'static str {
        "mysql"
    }

    fn quote_identifier_char(&self) -> char {
        '`'
    }

    fn parameter_placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }

    fn datetime_format(&self) -> &'static str {
        "%Y-%m-%d %H:%M:%S"
    }

    fn get_type_declaration(&self, sql_type: &SqlType) -> String {
        match sql_type {
            SqlType::SmallInt => "SMALLINT".to_string(),
            SqlType::Integer => "INT".to_string(),
            SqlType::BigInt => "BIGINT".to_string(),
            SqlType::Float => "FLOAT".to_string(),
            SqlType::Double => "DOUBLE".to_string(),
            SqlType::Decimal { precision, scale } => format!("DECIMAL({}, {})", precision, scale),
            SqlType::Char { length } => format!("CHAR({})", length),
            SqlType::Varchar { length } => format!("VARCHAR({})", length),
            SqlType::Text => "TEXT".to_string(),
            SqlType::Binary { length } => format!("BINARY({})", length),
            SqlType::VarBinary { length } => format!("VARBINARY({})", length),
            SqlType::Blob => "LONGBLOB".to_string(),
            SqlType::Boolean => "TINYINT(1)".to_string(), // MySQL uses TINYINT for boolean
            SqlType::Date => "DATE".to_string(),
            SqlType::Time { precision } => match precision {
                Some(p) => format!("TIME({})", p),
                None => "TIME".to_string(),
            },
            SqlType::Timestamp { precision } => match precision {
                Some(p) => format!("DATETIME({})", p),
                None => "DATETIME".to_string(),
            },
            SqlType::TimestampTz { precision } => match precision {
                Some(p) => format!("TIMESTAMP({})", p),
                None => "TIMESTAMP".to_string(),
            },
            SqlType::Uuid => "CHAR(36)".to_string(), // MySQL doesn't have native UUID
            SqlType::Json => "JSON".to_string(),
            SqlType::Serial => "INT AUTO_INCREMENT".to_string(),
            SqlType::BigSerial => "BIGINT AUTO_INCREMENT".to_string(),
        }
    }

    fn get_column_declaration(&self, column: &Column) -> String {
        let type_decl = self.get_type_declaration(&column.sql_type);

        // Handle AUTO_INCREMENT separately for MySQL
        let (base_type, has_auto_inc) = if type_decl.ends_with(" AUTO_INCREMENT") {
            (type_decl.trim_end_matches(" AUTO_INCREMENT").to_string(), true)
        } else {
            (type_decl, column.auto_increment)
        };

        let mut sql = format!(
            "{} {}",
            self.quote_identifier(&column.name),
            base_type
        );

        if !column.nullable {
            sql.push_str(" NOT NULL");
        }

        if has_auto_inc {
            sql.push_str(" AUTO_INCREMENT");
        }

        if let Some(ref default) = column.default {
            sql.push_str(" DEFAULT ");
            sql.push_str(default);
        }

        sql
    }

    fn get_drop_index_sql(&self, index_name: &str, table_name: &str) -> String {
        // MySQL requires table name for DROP INDEX
        format!(
            "DROP INDEX {} ON {}",
            self.quote_identifier(index_name),
            self.quote_identifier(table_name)
        )
    }

    fn get_list_tables_sql(&self) -> &'static str {
        "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE'"
    }

    fn get_list_columns_sql(&self, table_name: &str) -> String {
        format!(
            "SELECT column_name, data_type, is_nullable, column_default, character_maximum_length, numeric_precision, numeric_scale, extra \
             FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = '{}' ORDER BY ordinal_position",
            table_name
        )
    }

    fn get_list_indexes_sql(&self, table_name: &str) -> String {
        format!(
            "SELECT index_name, column_name, non_unique \
             FROM information_schema.statistics WHERE table_schema = DATABASE() AND table_name = '{}' \
             ORDER BY index_name, seq_in_index",
            table_name
        )
    }

    fn get_list_foreign_keys_sql(&self, table_name: &str) -> String {
        format!(
            "SELECT constraint_name, column_name, referenced_table_name, referenced_column_name \
             FROM information_schema.key_column_usage \
             WHERE table_schema = DATABASE() AND table_name = '{}' AND referenced_table_name IS NOT NULL",
            table_name
        )
    }
}

/// SQLite platform
pub struct SqlitePlatform;

impl Platform for SqlitePlatform {
    fn name(&self) -> &'static str {
        "sqlite"
    }

    fn quote_identifier_char(&self) -> char {
        '"'
    }

    fn supports_returning(&self) -> bool {
        true // SQLite 3.35+ supports RETURNING
    }

    fn parameter_placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }

    fn get_type_declaration(&self, sql_type: &SqlType) -> String {
        // SQLite uses dynamic typing with type affinity
        match sql_type {
            SqlType::SmallInt | SqlType::Integer | SqlType::BigInt => "INTEGER".to_string(),
            SqlType::Float | SqlType::Double => "REAL".to_string(),
            SqlType::Decimal { .. } => "REAL".to_string(), // SQLite doesn't have DECIMAL
            SqlType::Char { .. } | SqlType::Varchar { .. } | SqlType::Text => "TEXT".to_string(),
            SqlType::Binary { .. } | SqlType::VarBinary { .. } | SqlType::Blob => "BLOB".to_string(),
            SqlType::Boolean => "INTEGER".to_string(), // SQLite uses 0/1 for boolean
            SqlType::Date | SqlType::Time { .. } | SqlType::Timestamp { .. } | SqlType::TimestampTz { .. } => {
                "TEXT".to_string() // SQLite stores dates as TEXT
            }
            SqlType::Uuid => "TEXT".to_string(),
            SqlType::Json => "TEXT".to_string(), // SQLite has JSON functions but stores as TEXT
            SqlType::Serial | SqlType::BigSerial => "INTEGER".to_string(),
        }
    }

    fn get_column_declaration(&self, column: &Column) -> String {
        let mut sql = format!(
            "{} {}",
            self.quote_identifier(&column.name),
            self.get_type_declaration(&column.sql_type)
        );

        // SQLite PRIMARY KEY implies AUTOINCREMENT for INTEGER
        if column.auto_increment {
            sql.push_str(" PRIMARY KEY AUTOINCREMENT");
        } else {
            if !column.nullable {
                sql.push_str(" NOT NULL");
            }

            if let Some(ref default) = column.default {
                sql.push_str(" DEFAULT ");
                sql.push_str(default);
            }
        }

        sql
    }

    fn get_create_table_sql(&self, table: &Table) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.quote_identifier(&table.name));

        // Check if we have an auto-increment column (which becomes the PK in SQLite)
        let has_auto_inc = table.columns.iter().any(|c| c.auto_increment);

        // Columns
        let column_defs: Vec<String> = table
            .columns
            .iter()
            .map(|col| format!("    {}", self.get_column_declaration(col)))
            .collect();
        sql.push_str(&column_defs.join(",\n"));

        // Primary key (only if no auto-increment column, since that already has PK)
        if !has_auto_inc {
            if let Some(pk_cols) = table.primary_key_columns() {
                let pk_col_names: Vec<String> = pk_cols
                    .iter()
                    .map(|c| self.quote_identifier(c))
                    .collect();
                sql.push_str(&format!(",\n    PRIMARY KEY ({})", pk_col_names.join(", ")));
            }
        }

        // Unique indexes as constraints
        for index in &table.indexes {
            if index.unique && !index.primary {
                let col_names: Vec<String> = index
                    .columns
                    .iter()
                    .map(|c| self.quote_identifier(c))
                    .collect();
                sql.push_str(&format!(",\n    UNIQUE ({})", col_names.join(", ")));
            }
        }

        // Foreign keys
        for fk in &table.foreign_keys {
            let local_cols: Vec<String> = fk
                .local_columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            let foreign_cols: Vec<String> = fk
                .foreign_columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();

            sql.push_str(&format!(
                ",\n    FOREIGN KEY ({}) REFERENCES {} ({})",
                local_cols.join(", "),
                self.quote_identifier(&fk.foreign_table),
                foreign_cols.join(", ")
            ));

            if fk.on_delete != super::types::ForeignKeyAction::NoAction {
                sql.push_str(&format!(" ON DELETE {}", fk.on_delete.as_sql()));
            }
            if fk.on_update != super::types::ForeignKeyAction::NoAction {
                sql.push_str(&format!(" ON UPDATE {}", fk.on_update.as_sql()));
            }
        }

        sql.push_str("\n)");
        sql
    }

    fn release_savepoint_sql(&self, name: &str) -> String {
        // SQLite uses RELEASE without SAVEPOINT keyword
        format!("RELEASE {}", self.quote_identifier(name))
    }

    fn get_list_tables_sql(&self) -> &'static str {
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
    }

    fn get_list_columns_sql(&self, table_name: &str) -> String {
        format!("PRAGMA table_info({})", self.quote_identifier(table_name))
    }

    fn get_list_indexes_sql(&self, table_name: &str) -> String {
        format!("PRAGMA index_list({})", self.quote_identifier(table_name))
    }

    fn get_list_foreign_keys_sql(&self, table_name: &str) -> String {
        format!("PRAGMA foreign_key_list({})", self.quote_identifier(table_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::types::{Column, ForeignKey, ForeignKeyAction, Index, SqlType, Table};

    #[test]
    fn test_postgres_quote_identifier() {
        let platform = PostgresPlatform;
        assert_eq!(platform.quote_identifier("users"), "\"users\"");
        assert_eq!(platform.quote_identifier("user\"name"), "\"user\"\"name\"");
    }

    #[test]
    fn test_mysql_quote_identifier() {
        let platform = MySqlPlatform;
        assert_eq!(platform.quote_identifier("users"), "`users`");
    }

    #[test]
    fn test_postgres_parameter() {
        let platform = PostgresPlatform;
        assert_eq!(platform.parameter_placeholder(0), "$1");
        assert_eq!(platform.parameter_placeholder(1), "$2");
    }

    #[test]
    fn test_mysql_parameter() {
        let platform = MySqlPlatform;
        assert_eq!(platform.parameter_placeholder(0), "?");
        assert_eq!(platform.parameter_placeholder(1), "?");
    }

    #[test]
    fn test_limit_offset() {
        let platform = PostgresPlatform;
        assert_eq!(platform.limit_offset_sql(Some(10), None), " LIMIT 10");
        assert_eq!(platform.limit_offset_sql(Some(10), Some(5)), " LIMIT 10 OFFSET 5");
        assert_eq!(platform.limit_offset_sql(None, Some(5)), " OFFSET 5");
    }

    // Type declaration tests
    #[test]
    fn test_postgres_type_declarations() {
        let platform = PostgresPlatform;
        assert_eq!(platform.get_type_declaration(&SqlType::Integer), "INTEGER");
        assert_eq!(platform.get_type_declaration(&SqlType::BigInt), "BIGINT");
        assert_eq!(platform.get_type_declaration(&SqlType::varchar(255)), "VARCHAR(255)");
        assert_eq!(platform.get_type_declaration(&SqlType::Text), "TEXT");
        assert_eq!(platform.get_type_declaration(&SqlType::decimal(10, 2)), "NUMERIC(10, 2)");
        assert_eq!(platform.get_type_declaration(&SqlType::Boolean), "BOOLEAN");
        assert_eq!(platform.get_type_declaration(&SqlType::Uuid), "UUID");
        assert_eq!(platform.get_type_declaration(&SqlType::Json), "JSONB");
        assert_eq!(platform.get_type_declaration(&SqlType::Serial), "SERIAL");
        assert_eq!(platform.get_type_declaration(&SqlType::TimestampTz { precision: None }), "TIMESTAMP WITH TIME ZONE");
    }

    #[test]
    fn test_mysql_type_declarations() {
        let platform = MySqlPlatform;
        assert_eq!(platform.get_type_declaration(&SqlType::Integer), "INT");
        assert_eq!(platform.get_type_declaration(&SqlType::Boolean), "TINYINT(1)");
        assert_eq!(platform.get_type_declaration(&SqlType::Uuid), "CHAR(36)");
        assert_eq!(platform.get_type_declaration(&SqlType::Serial), "INT AUTO_INCREMENT");
        assert_eq!(platform.get_type_declaration(&SqlType::Blob), "LONGBLOB");
    }

    #[test]
    fn test_sqlite_type_declarations() {
        let platform = SqlitePlatform;
        // SQLite uses type affinity
        assert_eq!(platform.get_type_declaration(&SqlType::Integer), "INTEGER");
        assert_eq!(platform.get_type_declaration(&SqlType::BigInt), "INTEGER");
        assert_eq!(platform.get_type_declaration(&SqlType::varchar(255)), "TEXT");
        assert_eq!(platform.get_type_declaration(&SqlType::Boolean), "INTEGER");
        assert_eq!(platform.get_type_declaration(&SqlType::Uuid), "TEXT");
        assert_eq!(platform.get_type_declaration(&SqlType::Date), "TEXT");
    }

    // DDL generation tests
    #[test]
    fn test_postgres_create_table() {
        let platform = PostgresPlatform;
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Serial).not_null())
            .column(Column::new("name", SqlType::varchar(100)).not_null())
            .column(Column::new("email", SqlType::varchar(255)))
            .index(Index::primary(vec!["id".to_string()]));

        let sql = platform.get_create_table_sql(&table);
        assert!(sql.contains("CREATE TABLE \"users\""));
        assert!(sql.contains("\"id\" SERIAL NOT NULL"));
        assert!(sql.contains("\"name\" VARCHAR(100) NOT NULL"));
        assert!(sql.contains("\"email\" VARCHAR(255)"));
        assert!(sql.contains("PRIMARY KEY (\"id\")"));
    }

    #[test]
    fn test_mysql_create_table() {
        let platform = MySqlPlatform;
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Serial).not_null())
            .column(Column::new("name", SqlType::varchar(100)).not_null())
            .index(Index::primary(vec!["id".to_string()]));

        let sql = platform.get_create_table_sql(&table);
        assert!(sql.contains("CREATE TABLE `users`"));
        assert!(sql.contains("`id` INT NOT NULL AUTO_INCREMENT"));
        assert!(sql.contains("`name` VARCHAR(100) NOT NULL"));
    }

    #[test]
    fn test_sqlite_create_table() {
        let platform = SqlitePlatform;
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Integer).not_null().auto_increment())
            .column(Column::new("name", SqlType::varchar(100)).not_null())
            .index(Index::primary(vec!["id".to_string()]));

        let sql = platform.get_create_table_sql(&table);
        assert!(sql.contains("CREATE TABLE \"users\""));
        assert!(sql.contains("\"id\" INTEGER PRIMARY KEY AUTOINCREMENT"));
        assert!(sql.contains("\"name\" TEXT NOT NULL"));
        // Should not have separate PRIMARY KEY since AUTOINCREMENT implies it
        assert!(!sql.contains("PRIMARY KEY (\"id\")"));
    }

    #[test]
    fn test_create_table_with_foreign_key() {
        let platform = PostgresPlatform;
        let table = Table::new("posts")
            .column(Column::new("id", SqlType::Serial).not_null())
            .column(Column::new("user_id", SqlType::Integer).not_null())
            .column(Column::new("title", SqlType::varchar(200)).not_null())
            .index(Index::primary(vec!["id".to_string()]))
            .foreign_key(ForeignKey {
                name: "fk_posts_user".to_string(),
                local_columns: vec!["user_id".to_string()],
                foreign_table: "users".to_string(),
                foreign_columns: vec!["id".to_string()],
                on_delete: ForeignKeyAction::Cascade,
                on_update: ForeignKeyAction::NoAction,
            });

        let sql = platform.get_create_table_sql(&table);
        assert!(sql.contains("FOREIGN KEY (\"user_id\") REFERENCES \"users\" (\"id\")"));
        assert!(sql.contains("ON DELETE CASCADE"));
    }

    #[test]
    fn test_drop_table() {
        let platform = PostgresPlatform;
        assert_eq!(platform.get_drop_table_sql("users"), "DROP TABLE \"users\"");
        assert_eq!(
            platform.get_drop_table_if_exists_sql("users"),
            "DROP TABLE IF EXISTS \"users\""
        );
    }

    #[test]
    fn test_create_index() {
        let platform = PostgresPlatform;
        let index = Index::new("idx_users_email", vec!["email".to_string()]);
        let sql = platform.get_create_index_sql("users", &index);
        assert_eq!(sql, "CREATE INDEX \"idx_users_email\" ON \"users\" (\"email\")");

        let unique_index = Index::unique("idx_users_email_unique", vec!["email".to_string()]);
        let sql = platform.get_create_index_sql("users", &unique_index);
        assert_eq!(sql, "CREATE UNIQUE INDEX \"idx_users_email_unique\" ON \"users\" (\"email\")");
    }

    // Schema introspection SQL tests
    #[test]
    fn test_postgres_introspection_sql() {
        let platform = PostgresPlatform;
        assert!(platform.get_list_tables_sql().contains("information_schema.tables"));
        assert!(platform.get_list_columns_sql("users").contains("information_schema.columns"));
        assert!(platform.get_list_indexes_sql("users").contains("pg_index"));
    }

    #[test]
    fn test_mysql_introspection_sql() {
        let platform = MySqlPlatform;
        assert!(platform.get_list_tables_sql().contains("information_schema.tables"));
        assert!(platform.get_list_tables_sql().contains("DATABASE()"));
    }

    #[test]
    fn test_sqlite_introspection_sql() {
        let platform = SqlitePlatform;
        assert!(platform.get_list_tables_sql().contains("sqlite_master"));
        assert!(platform.get_list_columns_sql("users").contains("PRAGMA table_info"));
        assert!(platform.get_list_indexes_sql("users").contains("PRAGMA index_list"));
    }

    #[test]
    fn test_sqlite_release_savepoint() {
        let platform = SqlitePlatform;
        // SQLite uses RELEASE without SAVEPOINT keyword
        assert_eq!(platform.release_savepoint_sql("sp1"), "RELEASE \"sp1\"");

        let pg = PostgresPlatform;
        assert_eq!(pg.release_savepoint_sql("sp1"), "RELEASE SAVEPOINT \"sp1\"");
    }
}
