//! Platform trait for SQL dialect abstraction

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
