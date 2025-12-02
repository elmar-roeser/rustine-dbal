//! SQL value representation
//!
//! The [`SqlValue`] enum provides a type-safe representation of all values
//! that can be stored in or retrieved from a database.

use super::ParameterType;

/// A database value that can represent any SQL type
///
/// This is the intermediate representation used when converting between
/// Rust types and database types. It provides a unified interface for
/// handling all SQL values regardless of the underlying database platform.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SqlValue {
    /// SQL NULL value
    #[default]
    Null,

    /// Boolean value
    Bool(bool),

    /// Signed 8-bit integer
    I8(i8),

    /// Signed 16-bit integer (SMALLINT)
    I16(i16),

    /// Signed 32-bit integer (INT)
    I32(i32),

    /// Signed 64-bit integer (BIGINT)
    I64(i64),

    /// Unsigned 32-bit integer
    U32(u32),

    /// Unsigned 64-bit integer
    U64(u64),

    /// 32-bit floating point
    F32(f32),

    /// 64-bit floating point (DOUBLE)
    F64(f64),

    /// Text/String value (VARCHAR, TEXT, etc.)
    String(String),

    /// Binary data (BLOB, BYTEA, etc.)
    Bytes(Vec<u8>),

    /// Date value (year, month, day)
    #[cfg(feature = "chrono")]
    Date(chrono::NaiveDate),

    /// Time value (hour, minute, second, nanosecond)
    #[cfg(feature = "chrono")]
    Time(chrono::NaiveTime),

    /// `DateTime` value without timezone
    #[cfg(feature = "chrono")]
    DateTime(chrono::NaiveDateTime),

    /// `DateTime` value with UTC timezone
    #[cfg(feature = "chrono")]
    DateTimeUtc(chrono::DateTime<chrono::Utc>),

    /// UUID value
    #[cfg(feature = "uuid")]
    Uuid(uuid::Uuid),

    /// JSON value
    #[cfg(feature = "json")]
    Json(serde_json::Value),

    /// Decimal value for precise numeric storage
    #[cfg(feature = "decimal")]
    Decimal(rust_decimal::Decimal),
}

impl SqlValue {
    /// Check if this value is NULL
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get the parameter type for this value
    #[must_use]
    pub const fn param_type(&self) -> ParameterType {
        match self {
            Self::Null => ParameterType::Null,
            Self::Bool(_) => ParameterType::Boolean,
            Self::I8(_) | Self::I16(_) | Self::I32(_) | Self::I64(_) => ParameterType::Integer,
            Self::U32(_) | Self::U64(_) => ParameterType::Integer,
            Self::F32(_) | Self::F64(_) => ParameterType::String, // Often bound as string for precision
            Self::String(_) => ParameterType::String,
            Self::Bytes(_) => ParameterType::Binary,
            #[cfg(feature = "chrono")]
            Self::Date(_) | Self::Time(_) | Self::DateTime(_) | Self::DateTimeUtc(_) => {
                ParameterType::String
            }
            #[cfg(feature = "uuid")]
            Self::Uuid(_) => ParameterType::String,
            #[cfg(feature = "json")]
            Self::Json(_) => ParameterType::String,
            #[cfg(feature = "decimal")]
            Self::Decimal(_) => ParameterType::String,
        }
    }

    /// Try to get as bool
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            Self::I8(i) => Some(*i != 0),
            Self::I16(i) => Some(*i != 0),
            Self::I32(i) => Some(*i != 0),
            Self::I64(i) => Some(*i != 0),
            _ => None,
        }
    }

    /// Try to get as i32
    #[must_use]
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Self::I8(i) => Some(i32::from(*i)),
            Self::I16(i) => Some(i32::from(*i)),
            Self::I32(i) => Some(*i),
            Self::Bool(b) => Some(i32::from(*b)),
            _ => None,
        }
    }

    /// Try to get as i64
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I8(i) => Some(i64::from(*i)),
            Self::I16(i) => Some(i64::from(*i)),
            Self::I32(i) => Some(i64::from(*i)),
            Self::I64(i) => Some(*i),
            Self::U32(u) => Some(i64::from(*u)),
            Self::Bool(b) => Some(i64::from(*b)),
            _ => None,
        }
    }

    /// Try to get as f64
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F32(f) => Some(f64::from(*f)),
            Self::F64(f) => Some(*f),
            Self::I8(i) => Some(f64::from(*i)),
            Self::I16(i) => Some(f64::from(*i)),
            Self::I32(i) => Some(f64::from(*i)),
            Self::I64(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Try to get as string reference
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as bytes reference
    #[must_use]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            Self::String(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    /// Convert to String representation
    #[must_use]
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            Self::I8(i) => Some(i.to_string()),
            Self::I16(i) => Some(i.to_string()),
            Self::I32(i) => Some(i.to_string()),
            Self::I64(i) => Some(i.to_string()),
            Self::U32(u) => Some(u.to_string()),
            Self::U64(u) => Some(u.to_string()),
            Self::F32(f) => Some(f.to_string()),
            Self::F64(f) => Some(f.to_string()),
            Self::Bool(b) => Some(b.to_string()),
            #[cfg(feature = "uuid")]
            Self::Uuid(u) => Some(u.to_string()),
            #[cfg(feature = "decimal")]
            Self::Decimal(d) => Some(d.to_string()),
            _ => None,
        }
    }

    /// Get as UUID
    #[cfg(feature = "uuid")]
    #[must_use]
    pub const fn as_uuid(&self) -> Option<&uuid::Uuid> {
        match self {
            Self::Uuid(u) => Some(u),
            _ => None,
        }
    }

    /// Get as JSON value
    #[cfg(feature = "json")]
    #[must_use]
    pub const fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json(j) => Some(j),
            _ => None,
        }
    }

    /// Get as `NaiveDate`
    #[cfg(feature = "chrono")]
    #[must_use]
    pub const fn as_date(&self) -> Option<&chrono::NaiveDate> {
        match self {
            Self::Date(d) => Some(d),
            _ => None,
        }
    }

    /// Get as `NaiveTime`
    #[cfg(feature = "chrono")]
    #[must_use]
    pub const fn as_time(&self) -> Option<&chrono::NaiveTime> {
        match self {
            Self::Time(t) => Some(t),
            _ => None,
        }
    }

    /// Get as `NaiveDateTime`
    #[cfg(feature = "chrono")]
    #[must_use]
    pub const fn as_datetime(&self) -> Option<&chrono::NaiveDateTime> {
        match self {
            Self::DateTime(dt) => Some(dt),
            _ => None,
        }
    }

    /// Get as Decimal
    #[cfg(feature = "decimal")]
    #[must_use]
    pub const fn as_decimal(&self) -> Option<&rust_decimal::Decimal> {
        match self {
            Self::Decimal(d) => Some(d),
            _ => None,
        }
    }
}

impl std::fmt::Display for SqlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::I8(i) => write!(f, "{i}"),
            Self::I16(i) => write!(f, "{i}"),
            Self::I32(i) => write!(f, "{i}"),
            Self::I64(i) => write!(f, "{i}"),
            Self::U32(u) => write!(f, "{u}"),
            Self::U64(u) => write!(f, "{u}"),
            Self::F32(n) => write!(f, "{n}"),
            Self::F64(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "'{}'", s.replace('\'', "''")),
            Self::Bytes(b) => write!(f, "0x{}", hex_encode(b)),
            #[cfg(feature = "chrono")]
            Self::Date(d) => write!(f, "'{d}'"),
            #[cfg(feature = "chrono")]
            Self::Time(t) => write!(f, "'{t}'"),
            #[cfg(feature = "chrono")]
            Self::DateTime(dt) => write!(f, "'{dt}'"),
            #[cfg(feature = "chrono")]
            Self::DateTimeUtc(dt) => write!(f, "'{dt}'"),
            #[cfg(feature = "uuid")]
            Self::Uuid(u) => write!(f, "'{u}'"),
            #[cfg(feature = "json")]
            Self::Json(j) => write!(f, "'{j}'"),
            #[cfg(feature = "decimal")]
            Self::Decimal(d) => write!(f, "{d}"),
        }
    }
}

/// Simple hex encoding for bytes display
fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

// Convenient From implementations
impl From<bool> for SqlValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i8> for SqlValue {
    fn from(value: i8) -> Self {
        Self::I8(value)
    }
}

impl From<i16> for SqlValue {
    fn from(value: i16) -> Self {
        Self::I16(value)
    }
}

impl From<i32> for SqlValue {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<i64> for SqlValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u32> for SqlValue {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<u64> for SqlValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<f32> for SqlValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<f64> for SqlValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<String> for SqlValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for SqlValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<u8>> for SqlValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<&[u8]> for SqlValue {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(value.to_vec())
    }
}

impl<T> From<Option<T>> for SqlValue
where
    T: Into<Self>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::Null,
        }
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDate> for SqlValue {
    fn from(value: chrono::NaiveDate) -> Self {
        Self::Date(value)
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveTime> for SqlValue {
    fn from(value: chrono::NaiveTime) -> Self {
        Self::Time(value)
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDateTime> for SqlValue {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Self::DateTime(value)
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::DateTime<chrono::Utc>> for SqlValue {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self::DateTimeUtc(value)
    }
}

#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for SqlValue {
    fn from(value: uuid::Uuid) -> Self {
        Self::Uuid(value)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Value> for SqlValue {
    fn from(value: serde_json::Value) -> Self {
        Self::Json(value)
    }
}

#[cfg(feature = "decimal")]
impl From<rust_decimal::Decimal> for SqlValue {
    fn from(value: rust_decimal::Decimal) -> Self {
        Self::Decimal(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_value_null() {
        let value = SqlValue::Null;
        assert!(value.is_null());
        assert_eq!(value.param_type(), ParameterType::Null);
        assert_eq!(value.to_string(), "NULL");
    }

    #[test]
    fn test_sql_value_from_primitives() {
        assert_eq!(SqlValue::from(true), SqlValue::Bool(true));
        assert_eq!(SqlValue::from(42i32), SqlValue::I32(42));
        assert_eq!(SqlValue::from(42i64), SqlValue::I64(42));
        assert_eq!(SqlValue::from(3.14f64), SqlValue::F64(3.14));
        assert_eq!(
            SqlValue::from("hello"),
            SqlValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_sql_value_option() {
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;

        assert_eq!(SqlValue::from(some_val), SqlValue::I32(42));
        assert_eq!(SqlValue::from(none_val), SqlValue::Null);
    }

    #[test]
    fn test_sql_value_conversions() {
        let value = SqlValue::I32(42);
        assert_eq!(value.as_i32(), Some(42));
        assert_eq!(value.as_i64(), Some(42));
        assert_eq!(value.as_f64(), Some(42.0));
        assert_eq!(value.as_bool(), Some(true)); // non-zero i32 converts to true
        assert_eq!(SqlValue::I32(0).as_bool(), Some(false)); // zero converts to false

        let bool_val = SqlValue::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_i64(), Some(1));

        // String doesn't convert to bool via as_bool
        assert_eq!(SqlValue::String("test".into()).as_bool(), None);
    }

    #[test]
    fn test_sql_value_display() {
        assert_eq!(SqlValue::I32(42).to_string(), "42");
        assert_eq!(SqlValue::String("test".into()).to_string(), "'test'");
        assert_eq!(
            SqlValue::String("it's".into()).to_string(),
            "'it''s'"
        ); // Escaped single quote
        assert_eq!(SqlValue::Bool(true).to_string(), "true");
        assert_eq!(SqlValue::Bytes(vec![0xDE, 0xAD]).to_string(), "0xdead");
    }

    #[test]
    fn test_sql_value_into_string() {
        assert_eq!(SqlValue::I32(42).into_string(), Some("42".to_string()));
        assert_eq!(
            SqlValue::String("hello".into()).into_string(),
            Some("hello".to_string())
        );
        assert_eq!(SqlValue::Bool(true).into_string(), Some("true".to_string()));
        assert_eq!(SqlValue::Null.into_string(), None);
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_sql_value_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let value = SqlValue::from(uuid);
        assert_eq!(value.as_uuid(), Some(&uuid));
        assert_eq!(value.param_type(), ParameterType::String);
    }
}
