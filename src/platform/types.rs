//! SQL type definitions for schema operations
//!
//! These types represent SQL column types with their parameters
//! for DDL generation and schema introspection.

/// SQL column type with optional parameters
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SqlType {
    // Integer types
    /// SMALLINT (2 bytes)
    SmallInt,
    /// INTEGER (4 bytes)
    #[default]
    Integer,
    /// BIGINT (8 bytes)
    BigInt,

    // Floating point
    /// REAL/FLOAT (4 bytes)
    Float,
    /// DOUBLE PRECISION (8 bytes)
    Double,

    // Exact numeric
    /// DECIMAL/NUMERIC with precision and scale
    Decimal {
        /// Total number of digits
        precision: u8,
        /// Number of digits after decimal point
        scale: u8,
    },

    // String types
    /// CHAR(n) - fixed length
    Char {
        /// Fixed string length
        length: u32,
    },
    /// VARCHAR(n) - variable length
    Varchar {
        /// Maximum string length
        length: u32,
    },
    /// TEXT - unlimited length
    Text,

    // Binary types
    /// BINARY(n) - fixed length binary
    Binary {
        /// Fixed binary length
        length: u32,
    },
    /// VARBINARY(n) / BYTEA - variable length binary
    VarBinary {
        /// Maximum binary length
        length: u32,
    },
    /// BLOB - large binary object
    Blob,

    // Boolean
    /// BOOLEAN
    Boolean,

    // Date/Time types
    /// DATE
    Date,
    /// TIME with optional precision
    Time {
        /// Fractional seconds precision
        precision: Option<u8>,
    },
    /// TIMESTAMP/DATETIME with optional precision
    Timestamp {
        /// Fractional seconds precision
        precision: Option<u8>,
    },
    /// TIMESTAMP WITH TIME ZONE
    TimestampTz {
        /// Fractional seconds precision
        precision: Option<u8>,
    },

    // Special types
    /// UUID (native or CHAR(36))
    Uuid,
    /// JSON/JSONB
    Json,
    /// Auto-incrementing integer (SERIAL, AUTOINCREMENT, etc.)
    Serial,
    /// Auto-incrementing big integer
    BigSerial,
}

impl SqlType {
    /// Create a VARCHAR with the given length
    #[must_use]
    pub const fn varchar(length: u32) -> Self {
        Self::Varchar { length }
    }

    /// Create a CHAR with the given length
    #[must_use]
    pub const fn char(length: u32) -> Self {
        Self::Char { length }
    }

    /// Create a DECIMAL with precision and scale
    #[must_use]
    pub const fn decimal(precision: u8, scale: u8) -> Self {
        Self::Decimal { precision, scale }
    }

    /// Create a TIMESTAMP with optional precision
    #[must_use]
    pub const fn timestamp(precision: Option<u8>) -> Self {
        Self::Timestamp { precision }
    }

    /// Check if this type is a string type
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::Char { .. } | Self::Varchar { .. } | Self::Text)
    }

    /// Check if this type is a numeric type
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::SmallInt
                | Self::Integer
                | Self::BigInt
                | Self::Float
                | Self::Double
                | Self::Decimal { .. }
                | Self::Serial
                | Self::BigSerial
        )
    }

    /// Check if this type is a date/time type
    #[must_use]
    pub const fn is_datetime(&self) -> bool {
        matches!(
            self,
            Self::Date
                | Self::Time { .. }
                | Self::Timestamp { .. }
                | Self::TimestampTz { .. }
        )
    }

    /// Check if this type is a binary type
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        matches!(
            self,
            Self::Binary { .. } | Self::VarBinary { .. } | Self::Blob
        )
    }

    /// Check if this type supports auto-increment
    #[must_use]
    pub const fn is_auto_increment(&self) -> bool {
        matches!(self, Self::Serial | Self::BigSerial)
    }
}

/// Column definition for schema operations
#[derive(Debug, Clone)]
pub struct Column {
    /// Column name
    pub name: String,
    /// SQL type
    pub sql_type: SqlType,
    /// Whether the column allows NULL values
    pub nullable: bool,
    /// Default value expression (as SQL string)
    pub default: Option<String>,
    /// Whether this column auto-increments
    pub auto_increment: bool,
    /// Column comment
    pub comment: Option<String>,
}

impl Column {
    /// Create a new column with the given name and type
    #[must_use]
    pub fn new(name: impl Into<String>, sql_type: SqlType) -> Self {
        Self {
            name: name.into(),
            sql_type,
            nullable: true,
            default: None,
            auto_increment: false,
            comment: None,
        }
    }

    /// Set the column as NOT NULL
    #[must_use]
    pub const fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Set a default value
    #[must_use]
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Set as auto-incrementing
    #[must_use]
    pub const fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// Set a comment
    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Index definition
#[derive(Debug, Clone)]
pub struct Index {
    /// Index name
    pub name: String,
    /// Column names in the index
    pub columns: Vec<String>,
    /// Whether this is a unique index
    pub unique: bool,
    /// Whether this is the primary key
    pub primary: bool,
}

impl Index {
    /// Create a new index
    #[must_use]
    pub fn new(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            unique: false,
            primary: false,
        }
    }

    /// Create a unique index
    #[must_use]
    pub fn unique(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            unique: true,
            primary: false,
        }
    }

    /// Create a primary key index
    #[must_use]
    pub const fn primary(columns: Vec<String>) -> Self {
        Self {
            name: String::new(), // Primary keys often don't have explicit names
            columns,
            unique: true,
            primary: true,
        }
    }
}

/// Foreign key definition
#[derive(Debug, Clone)]
pub struct ForeignKey {
    /// Constraint name
    pub name: String,
    /// Local column names
    pub local_columns: Vec<String>,
    /// Referenced table name
    pub foreign_table: String,
    /// Referenced column names
    pub foreign_columns: Vec<String>,
    /// ON DELETE action
    pub on_delete: ForeignKeyAction,
    /// ON UPDATE action
    pub on_update: ForeignKeyAction,
}

/// Foreign key referential action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ForeignKeyAction {
    /// No action (error if referenced row is modified)
    #[default]
    NoAction,
    /// Restrict (same as NO ACTION in most databases)
    Restrict,
    /// Cascade the change to referencing rows
    Cascade,
    /// Set referencing columns to NULL
    SetNull,
    /// Set referencing columns to their default value
    SetDefault,
}

impl ForeignKeyAction {
    /// Get the SQL representation
    #[must_use]
    pub const fn as_sql(&self) -> &'static str {
        match self {
            Self::NoAction => "NO ACTION",
            Self::Restrict => "RESTRICT",
            Self::Cascade => "CASCADE",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
        }
    }
}

/// Table definition for schema operations
#[derive(Debug, Clone)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Columns
    pub columns: Vec<Column>,
    /// Indexes (including primary key)
    pub indexes: Vec<Index>,
    /// Foreign keys
    pub foreign_keys: Vec<ForeignKey>,
    /// Table comment
    pub comment: Option<String>,
}

impl Table {
    /// Create a new table with the given name
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            comment: None,
        }
    }

    /// Add a column
    #[must_use]
    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Add an index
    #[must_use]
    pub fn index(mut self, index: Index) -> Self {
        self.indexes.push(index);
        self
    }

    /// Add a foreign key
    #[must_use]
    pub fn foreign_key(mut self, fk: ForeignKey) -> Self {
        self.foreign_keys.push(fk);
        self
    }

    /// Set a comment
    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Get the primary key columns
    #[must_use]
    pub fn primary_key_columns(&self) -> Option<&[String]> {
        self.indexes
            .iter()
            .find(|idx| idx.primary)
            .map(|idx| idx.columns.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_type_constructors() {
        assert_eq!(SqlType::varchar(255), SqlType::Varchar { length: 255 });
        assert_eq!(SqlType::char(10), SqlType::Char { length: 10 });
        assert_eq!(
            SqlType::decimal(10, 2),
            SqlType::Decimal {
                precision: 10,
                scale: 2
            }
        );
    }

    #[test]
    fn test_sql_type_categories() {
        assert!(SqlType::Varchar { length: 255 }.is_string());
        assert!(SqlType::Text.is_string());
        assert!(!SqlType::Integer.is_string());

        assert!(SqlType::Integer.is_numeric());
        assert!(SqlType::Decimal { precision: 10, scale: 2 }.is_numeric());
        assert!(!SqlType::Text.is_numeric());

        assert!(SqlType::Date.is_datetime());
        assert!(SqlType::Timestamp { precision: None }.is_datetime());
        assert!(!SqlType::Integer.is_datetime());
    }

    #[test]
    fn test_column_builder() {
        let col = Column::new("id", SqlType::Integer)
            .not_null()
            .auto_increment();

        assert_eq!(col.name, "id");
        assert!(!col.nullable);
        assert!(col.auto_increment);
    }

    #[test]
    fn test_table_builder() {
        let table = Table::new("users")
            .column(Column::new("id", SqlType::Serial).not_null())
            .column(Column::new("name", SqlType::varchar(100)).not_null())
            .column(Column::new("email", SqlType::varchar(255)))
            .index(Index::primary(vec!["id".to_string()]));

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.indexes.len(), 1);
        assert_eq!(
            table.primary_key_columns(),
            Some(vec!["id".to_string()].as_slice())
        );
    }

    #[test]
    fn test_foreign_key_action() {
        assert_eq!(ForeignKeyAction::Cascade.as_sql(), "CASCADE");
        assert_eq!(ForeignKeyAction::SetNull.as_sql(), "SET NULL");
    }
}
