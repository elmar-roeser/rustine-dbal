//! Error types for Rustine DBAL
//!
//! Provides a structured error hierarchy covering all database operations:
//! - Connection errors (lost, refused, authentication)
//! - Transaction errors (no active transaction, rollback-only, savepoint issues)
//! - Query errors (syntax, constraint violations, execution failures)
//! - Schema errors (table not found, column not found, introspection failures)
//! - Conversion errors (type conversion failures)

use thiserror::Error;

/// Result type alias using the Rustine Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for all Rustine operations
#[derive(Error, Debug)]
pub enum Error {
    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Transaction-related errors
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),

    /// Query execution errors
    #[error("Query error: {0}")]
    Query(#[from] QueryError),

    /// Schema-related errors
    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),

    /// Type conversion errors
    #[error("Conversion error: cannot convert {from_type} to {to_type}: {message}")]
    Conversion {
        /// The source type name
        from_type: &'static str,
        /// The target type name
        to_type: &'static str,
        /// Error message describing why conversion failed
        message: String,
    },

    /// Driver-level errors (wraps underlying database driver errors)
    #[error("Driver error: {message}")]
    Driver {
        /// Error message from the driver
        message: String,
        /// Optional underlying error source
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Platform-specific errors
    #[error("Platform error: {0}")]
    Platform(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Connection-specific errors
#[derive(Error, Debug)]
pub enum ConnectionError {
    /// Connection to database was lost
    #[error("Connection lost")]
    Lost,

    /// Connection was refused by the server
    #[error("Connection refused: {0}")]
    Refused(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// Connection timeout
    #[error("Connection timeout after {0}ms")]
    Timeout(u64),

    /// Invalid connection string/URL
    #[error("Invalid connection URL: {0}")]
    InvalidUrl(String),

    /// Connection already closed
    #[error("Connection is closed")]
    Closed,

    /// Maximum connections reached
    #[error("Connection pool exhausted")]
    PoolExhausted,
}

/// Transaction-specific errors
#[derive(Error, Debug)]
pub enum TransactionError {
    /// No active transaction to commit/rollback
    #[error("No active transaction")]
    NoActiveTransaction,

    /// Transaction has been marked as rollback-only
    #[error("Transaction marked rollback-only")]
    RollbackOnly,

    /// Savepoints not supported by this platform
    #[error("Savepoints not supported")]
    SavepointsNotSupported,

    /// Savepoint not found
    #[error("Savepoint not found: {0}")]
    SavepointNotFound(String),

    /// Nested transactions not supported
    #[error("Nested transactions not supported")]
    NestedNotSupported,

    /// Transaction already started
    #[error("Transaction already active")]
    AlreadyActive,

    /// Commit failed
    #[error("Commit failed: {0}")]
    CommitFailed(String),

    /// Rollback failed
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}

/// Query execution errors
#[derive(Error, Debug)]
pub enum QueryError {
    /// SQL syntax error
    #[error("Syntax error: {message}")]
    Syntax {
        /// Error message describing the syntax error
        message: String,
        /// The SQL query that caused the error
        sql: Option<String>,
        /// Position in the SQL where the error occurred
        position: Option<u32>,
    },

    /// Constraint violation (unique, foreign key, check, etc.)
    #[error("{constraint_type} constraint violation: {message}")]
    ConstraintViolation {
        /// Type of constraint that was violated
        constraint_type: ConstraintType,
        /// Name of the constraint that was violated
        constraint_name: Option<String>,
        /// Error message describing the violation
        message: String,
    },

    /// Query execution failed
    #[error("Execution failed: {message}")]
    ExecutionFailed {
        /// Error message describing the failure
        message: String,
        /// The SQL query that failed
        sql: Option<String>,
    },

    /// Invalid parameter
    #[error("Invalid parameter '{name}': {message}")]
    InvalidParameter {
        /// Name of the invalid parameter
        name: String,
        /// Error message describing why the parameter is invalid
        message: String,
    },

    /// Missing parameter
    #[error("Missing parameter: {0}")]
    MissingParameter(String),

    /// Too many parameters
    #[error("Too many parameters: expected {expected}, got {actual}")]
    TooManyParameters {
        /// Number of parameters expected
        expected: usize,
        /// Number of parameters provided
        actual: usize,
    },

    /// Query timeout
    #[error("Query timeout after {0}ms")]
    Timeout(u64),

    /// Deadlock detected
    #[error("Deadlock detected")]
    Deadlock,

    /// Query was cancelled
    #[error("Query cancelled")]
    Cancelled,
}

/// Types of constraint violations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// Primary key constraint
    PrimaryKey,
    /// Unique constraint
    Unique,
    /// Foreign key constraint
    ForeignKey,
    /// Check constraint
    Check,
    /// Not null constraint
    NotNull,
    /// Unknown constraint type
    Unknown,
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PrimaryKey => write!(f, "Primary key"),
            Self::Unique => write!(f, "Unique"),
            Self::ForeignKey => write!(f, "Foreign key"),
            Self::Check => write!(f, "Check"),
            Self::NotNull => write!(f, "Not null"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Schema-related errors
#[derive(Error, Debug)]
pub enum SchemaError {
    /// Table not found
    #[error("Table not found: {0}")]
    TableNotFound(String),

    /// Column not found
    #[error("Column not found: {table}.{column}")]
    ColumnNotFound {
        /// Table name where the column was not found
        table: String,
        /// Column name that was not found
        column: String,
    },

    /// Index not found
    #[error("Index not found: {0}")]
    IndexNotFound(String),

    /// Schema introspection failed
    #[error("Introspection failed: {0}")]
    IntrospectionFailed(String),

    /// Invalid schema definition
    #[error("Invalid schema definition: {0}")]
    InvalidDefinition(String),

    /// Schema object already exists
    #[error("{object_type} already exists: {name}")]
    AlreadyExists {
        /// Type of schema object (e.g., "Table", "Index")
        object_type: &'static str,
        /// Name of the object that already exists
        name: String,
    },

    /// Unsupported schema operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

impl Error {
    /// Create a conversion error
    pub fn conversion(from_type: &'static str, to_type: &'static str, message: impl Into<String>) -> Self {
        Self::Conversion {
            from_type,
            to_type,
            message: message.into(),
        }
    }

    /// Create a driver error from any error source
    pub fn driver(message: impl Into<String>, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Driver {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a driver error without a source
    pub fn driver_message(message: impl Into<String>) -> Self {
        Self::Driver {
            message: message.into(),
            source: None,
        }
    }

    /// Create a platform error
    pub fn platform(message: impl Into<String>) -> Self {
        Self::Platform(message.into())
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    /// Check if this error is a connection error
    #[must_use]
    pub const fn is_connection_error(&self) -> bool {
        matches!(self, Self::Connection(_))
    }

    /// Check if this error is a transaction error
    #[must_use]
    pub const fn is_transaction_error(&self) -> bool {
        matches!(self, Self::Transaction(_))
    }

    /// Check if this error is a constraint violation
    #[must_use]
    pub const fn is_constraint_violation(&self) -> bool {
        matches!(self, Self::Query(QueryError::ConstraintViolation { .. }))
    }

    /// Check if this error is a deadlock
    #[must_use]
    pub const fn is_deadlock(&self) -> bool {
        matches!(self, Self::Query(QueryError::Deadlock))
    }

    /// Check if this error indicates the operation can be retried
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Connection(ConnectionError::Lost | ConnectionError::Timeout(_))
                | Self::Query(QueryError::Deadlock | QueryError::Timeout(_))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Connection(ConnectionError::Lost);
        assert_eq!(err.to_string(), "Connection error: Connection lost");

        let err = Error::conversion("i64", "u32", "value out of range");
        assert_eq!(
            err.to_string(),
            "Conversion error: cannot convert i64 to u32: value out of range"
        );
    }

    #[test]
    fn test_constraint_violation() {
        let err = Error::Query(QueryError::ConstraintViolation {
            constraint_type: ConstraintType::Unique,
            constraint_name: Some("users_email_key".to_string()),
            message: "duplicate key value".to_string(),
        });
        assert!(err.is_constraint_violation());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_retryable_errors() {
        assert!(Error::Connection(ConnectionError::Lost).is_retryable());
        assert!(Error::Connection(ConnectionError::Timeout(5000)).is_retryable());
        assert!(Error::Query(QueryError::Deadlock).is_retryable());
        assert!(!Error::Connection(ConnectionError::AuthFailed("bad password".into())).is_retryable());
    }
}
