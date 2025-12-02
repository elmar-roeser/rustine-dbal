//! `FromSql` trait for converting SQL values to Rust types
//!
//! This trait enables converting [`SqlValue`] instances back into
//! concrete Rust types.

use super::{Error, Result, SqlValue};

/// Trait for types that can be created from SQL values
///
/// Implement this trait for custom types that need to be read from database results.
///
/// # Example
///
/// ```rust
/// use rustine_dbal::{Error, Result, SqlValue};
/// use rustine_dbal::core::FromSql;
///
/// struct Money {
///     cents: i64,
/// }
///
/// impl FromSql for Money {
///     fn from_sql(value: SqlValue) -> Result<Self> {
///         match value {
///             SqlValue::I64(cents) => Ok(Money { cents }),
///             SqlValue::I32(cents) => Ok(Money { cents: cents as i64 }),
///             _ => Err(Error::conversion("SqlValue", "Money", "expected integer")),
///         }
///     }
/// }
/// ```
pub trait FromSql: Sized {
    /// Convert from a SQL value to this type
    ///
    /// # Errors
    ///
    /// Returns a conversion error if the SQL value cannot be converted to this type.
    fn from_sql(value: SqlValue) -> Result<Self>;

    /// Convert from a SQL value, allowing null as a valid input
    ///
    /// Returns `None` if the value is NULL, otherwise delegates to `from_sql`.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not NULL and cannot be converted.
    fn from_sql_nullable(value: SqlValue) -> Result<Option<Self>> {
        if value.is_null() {
            Ok(None)
        } else {
            Self::from_sql(value).map(Some)
        }
    }
}

impl FromSql for bool {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::Bool(b) => Ok(b),
            SqlValue::I8(i) => Ok(i != 0),
            SqlValue::I16(i) => Ok(i != 0),
            SqlValue::I32(i) => Ok(i != 0),
            SqlValue::I64(i) => Ok(i != 0),
            SqlValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "t" | "1" | "yes" | "on" => Ok(true),
                "false" | "f" | "0" | "no" | "off" => Ok(false),
                _ => Err(Error::conversion("String", "bool", format!("invalid boolean string: {s}"))),
            },
            _ => Err(Error::conversion(value_type_name(&value), "bool", "cannot convert to boolean")),
        }
    }
}

impl FromSql for i8 {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::I8(i) => Ok(i),
            SqlValue::I16(i) => i.try_into().map_err(|_| Error::conversion("i16", "i8", "value out of range")),
            SqlValue::I32(i) => i.try_into().map_err(|_| Error::conversion("i32", "i8", "value out of range")),
            SqlValue::I64(i) => i.try_into().map_err(|_| Error::conversion("i64", "i8", "value out of range")),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "i8", format!("invalid integer: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "i8", "cannot convert to i8")),
        }
    }
}

impl FromSql for i16 {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::I8(i) => Ok(Self::from(i)),
            SqlValue::I16(i) => Ok(i),
            SqlValue::I32(i) => i.try_into().map_err(|_| Error::conversion("i32", "i16", "value out of range")),
            SqlValue::I64(i) => i.try_into().map_err(|_| Error::conversion("i64", "i16", "value out of range")),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "i16", format!("invalid integer: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "i16", "cannot convert to i16")),
        }
    }
}

impl FromSql for i32 {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::I8(i) => Ok(Self::from(i)),
            SqlValue::I16(i) => Ok(Self::from(i)),
            SqlValue::I32(i) => Ok(i),
            SqlValue::I64(i) => i.try_into().map_err(|_| Error::conversion("i64", "i32", "value out of range")),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "i32", format!("invalid integer: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "i32", "cannot convert to i32")),
        }
    }
}

impl FromSql for i64 {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::I8(i) => Ok(Self::from(i)),
            SqlValue::I16(i) => Ok(Self::from(i)),
            SqlValue::I32(i) => Ok(Self::from(i)),
            SqlValue::I64(i) => Ok(i),
            SqlValue::U32(u) => Ok(Self::from(u)),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "i64", format!("invalid integer: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "i64", "cannot convert to i64")),
        }
    }
}

impl FromSql for u32 {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::I8(i) if i >= 0 => i.try_into().map_err(|_| Error::conversion("i8", "u32", "value out of range")),
            SqlValue::I16(i) if i >= 0 => i.try_into().map_err(|_| Error::conversion("i16", "u32", "value out of range")),
            SqlValue::I32(i) if i >= 0 => i.try_into().map_err(|_| Error::conversion("i32", "u32", "value out of range")),
            SqlValue::I64(i) => i.try_into().map_err(|_| Error::conversion("i64", "u32", "value out of range")),
            SqlValue::U32(u) => Ok(u),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "u32", format!("invalid integer: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "u32", "cannot convert to u32")),
        }
    }
}

impl FromSql for u64 {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::I8(i) if i >= 0 => i.try_into().map_err(|_| Error::conversion("i8", "u64", "value out of range")),
            SqlValue::I16(i) if i >= 0 => i.try_into().map_err(|_| Error::conversion("i16", "u64", "value out of range")),
            SqlValue::I32(i) if i >= 0 => i.try_into().map_err(|_| Error::conversion("i32", "u64", "value out of range")),
            SqlValue::I64(i) if i >= 0 => i.try_into().map_err(|_| Error::conversion("i64", "u64", "value out of range")),
            SqlValue::U32(u) => Ok(Self::from(u)),
            SqlValue::U64(u) => Ok(u),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "u64", format!("invalid integer: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "u64", "cannot convert to u64")),
        }
    }
}

impl FromSql for f32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::F32(f) => Ok(f),
            SqlValue::F64(f) => Ok(f as Self),
            SqlValue::I8(i) => Ok(Self::from(i)),
            SqlValue::I16(i) => Ok(Self::from(i)),
            SqlValue::I32(i) => Ok(i as Self),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "f32", format!("invalid float: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "f32", "cannot convert to f32")),
        }
    }
}

impl FromSql for f64 {
    #[allow(clippy::cast_precision_loss)]
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::F32(f) => Ok(Self::from(f)),
            SqlValue::F64(f) => Ok(f),
            SqlValue::I8(i) => Ok(Self::from(i)),
            SqlValue::I16(i) => Ok(Self::from(i)),
            SqlValue::I32(i) => Ok(Self::from(i)),
            SqlValue::I64(i) => Ok(i as Self),
            SqlValue::String(s) => s.parse().map_err(|_| Error::conversion("String", "f64", format!("invalid float: {s}"))),
            _ => Err(Error::conversion(value_type_name(&value), "f64", "cannot convert to f64")),
        }
    }
}

impl FromSql for String {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::String(s) => Ok(s),
            SqlValue::I8(i) => Ok(i.to_string()),
            SqlValue::I16(i) => Ok(i.to_string()),
            SqlValue::I32(i) => Ok(i.to_string()),
            SqlValue::I64(i) => Ok(i.to_string()),
            SqlValue::U32(u) => Ok(u.to_string()),
            SqlValue::U64(u) => Ok(u.to_string()),
            SqlValue::F32(f) => Ok(f.to_string()),
            SqlValue::F64(f) => Ok(f.to_string()),
            SqlValue::Bool(b) => Ok(b.to_string()),
            #[cfg(feature = "uuid")]
            SqlValue::Uuid(u) => Ok(u.to_string()),
            #[cfg(feature = "decimal")]
            SqlValue::Decimal(d) => Ok(d.to_string()),
            _ => Err(Error::conversion(value_type_name(&value), "String", "cannot convert to String")),
        }
    }
}

impl FromSql for Vec<u8> {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::Bytes(b) => Ok(b),
            SqlValue::String(s) => Ok(s.into_bytes()),
            _ => Err(Error::conversion(value_type_name(&value), "Vec<u8>", "cannot convert to bytes")),
        }
    }
}

impl<T: FromSql> FromSql for Option<T> {
    fn from_sql(value: SqlValue) -> Result<Self> {
        T::from_sql_nullable(value)
    }
}

impl FromSql for SqlValue {
    fn from_sql(value: SqlValue) -> Result<Self> {
        Ok(value)
    }
}

// Feature-gated implementations
#[cfg(feature = "chrono")]
impl FromSql for chrono::NaiveDate {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::Date(d) => Ok(d),
            SqlValue::String(s) => Self::parse_from_str(&s, "%Y-%m-%d")
                .map_err(|e| Error::conversion("String", "NaiveDate", e.to_string())),
            _ => Err(Error::conversion(value_type_name(&value), "NaiveDate", "cannot convert to date")),
        }
    }
}

#[cfg(feature = "chrono")]
impl FromSql for chrono::NaiveTime {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::Time(t) => Ok(t),
            SqlValue::String(s) => Self::parse_from_str(&s, "%H:%M:%S")
                .or_else(|_| Self::parse_from_str(&s, "%H:%M:%S%.f"))
                .map_err(|e| Error::conversion("String", "NaiveTime", e.to_string())),
            _ => Err(Error::conversion(value_type_name(&value), "NaiveTime", "cannot convert to time")),
        }
    }
}

#[cfg(feature = "chrono")]
impl FromSql for chrono::NaiveDateTime {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::DateTime(dt) => Ok(dt),
            SqlValue::String(s) => Self::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| Self::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f"))
                .or_else(|_| Self::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S"))
                .or_else(|_| Self::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f"))
                .map_err(|e| Error::conversion("String", "NaiveDateTime", e.to_string())),
            _ => Err(Error::conversion(value_type_name(&value), "NaiveDateTime", "cannot convert to datetime")),
        }
    }
}

#[cfg(feature = "chrono")]
impl FromSql for chrono::DateTime<chrono::Utc> {
    fn from_sql(value: SqlValue) -> Result<Self> {
        use chrono::TimeZone;

        match value {
            SqlValue::DateTimeUtc(dt) => Ok(dt),
            SqlValue::DateTime(dt) => Ok(chrono::Utc.from_utc_datetime(&dt)),
            SqlValue::String(s) => s.parse()
                .map_err(|e: chrono::ParseError| Error::conversion("String", "DateTime<Utc>", e.to_string())),
            _ => Err(Error::conversion(value_type_name(&value), "DateTime<Utc>", "cannot convert to datetime")),
        }
    }
}

#[cfg(feature = "uuid")]
impl FromSql for uuid::Uuid {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::Uuid(u) => Ok(u),
            SqlValue::String(s) => Self::parse_str(&s)
                .map_err(|e| Error::conversion("String", "Uuid", e.to_string())),
            SqlValue::Bytes(b) => Self::from_slice(&b)
                .map_err(|e| Error::conversion("Bytes", "Uuid", e.to_string())),
            _ => Err(Error::conversion(value_type_name(&value), "Uuid", "cannot convert to uuid")),
        }
    }
}

#[cfg(feature = "json")]
impl FromSql for serde_json::Value {
    fn from_sql(value: SqlValue) -> Result<Self> {
        match value {
            SqlValue::Json(j) => Ok(j),
            SqlValue::String(s) => serde_json::from_str(&s)
                .map_err(|e| Error::conversion("String", "serde_json::Value", e.to_string())),
            _ => Err(Error::conversion(value_type_name(&value), "serde_json::Value", "cannot convert to JSON")),
        }
    }
}

#[cfg(feature = "decimal")]
impl FromSql for rust_decimal::Decimal {
    fn from_sql(value: SqlValue) -> Result<Self> {
        use std::str::FromStr;

        match value {
            SqlValue::Decimal(d) => Ok(d),
            SqlValue::I8(i) => Ok(Self::from(i)),
            SqlValue::I16(i) => Ok(Self::from(i)),
            SqlValue::I32(i) => Ok(Self::from(i)),
            SqlValue::I64(i) => Ok(Self::from(i)),
            SqlValue::String(s) => Self::from_str(&s)
                .map_err(|e| Error::conversion("String", "Decimal", e.to_string())),
            _ => Err(Error::conversion(value_type_name(&value), "Decimal", "cannot convert to decimal")),
        }
    }
}

/// Get a human-readable type name for error messages
const fn value_type_name(value: &SqlValue) -> &'static str {
    match value {
        SqlValue::Null => "NULL",
        SqlValue::Bool(_) => "Bool",
        SqlValue::I8(_) => "i8",
        SqlValue::I16(_) => "i16",
        SqlValue::I32(_) => "i32",
        SqlValue::I64(_) => "i64",
        SqlValue::U32(_) => "u32",
        SqlValue::U64(_) => "u64",
        SqlValue::F32(_) => "f32",
        SqlValue::F64(_) => "f64",
        SqlValue::String(_) => "String",
        SqlValue::Bytes(_) => "Bytes",
        #[cfg(feature = "chrono")]
        SqlValue::Date(_) => "Date",
        #[cfg(feature = "chrono")]
        SqlValue::Time(_) => "Time",
        #[cfg(feature = "chrono")]
        SqlValue::DateTime(_) => "DateTime",
        #[cfg(feature = "chrono")]
        SqlValue::DateTimeUtc(_) => "DateTime<Utc>",
        #[cfg(feature = "uuid")]
        SqlValue::Uuid(_) => "Uuid",
        #[cfg(feature = "json")]
        SqlValue::Json(_) => "Json",
        #[cfg(feature = "decimal")]
        SqlValue::Decimal(_) => "Decimal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_sql_bool() {
        assert_eq!(bool::from_sql(SqlValue::Bool(true)).unwrap(), true);
        assert_eq!(bool::from_sql(SqlValue::I32(1)).unwrap(), true);
        assert_eq!(bool::from_sql(SqlValue::I32(0)).unwrap(), false);
        assert_eq!(bool::from_sql(SqlValue::String("true".into())).unwrap(), true);
        assert_eq!(bool::from_sql(SqlValue::String("false".into())).unwrap(), false);
    }

    #[test]
    fn test_from_sql_integers() {
        assert_eq!(i32::from_sql(SqlValue::I32(42)).unwrap(), 42);
        assert_eq!(i32::from_sql(SqlValue::I64(42)).unwrap(), 42);
        assert_eq!(i64::from_sql(SqlValue::I32(42)).unwrap(), 42);
        assert_eq!(i32::from_sql(SqlValue::String("42".into())).unwrap(), 42);
    }

    #[test]
    fn test_from_sql_integer_overflow() {
        let result = i8::from_sql(SqlValue::I64(1000));
        assert!(result.is_err());
    }

    #[test]
    fn test_from_sql_float() {
        assert_eq!(f64::from_sql(SqlValue::F64(3.14)).unwrap(), 3.14);
        assert_eq!(f64::from_sql(SqlValue::I32(42)).unwrap(), 42.0);
        assert_eq!(f64::from_sql(SqlValue::String("3.14".into())).unwrap(), 3.14);
    }

    #[test]
    fn test_from_sql_string() {
        assert_eq!(String::from_sql(SqlValue::String("hello".into())).unwrap(), "hello");
        assert_eq!(String::from_sql(SqlValue::I32(42)).unwrap(), "42");
        assert_eq!(String::from_sql(SqlValue::Bool(true)).unwrap(), "true");
    }

    #[test]
    fn test_from_sql_option() {
        assert_eq!(Option::<i32>::from_sql(SqlValue::Null).unwrap(), None);
        assert_eq!(Option::<i32>::from_sql(SqlValue::I32(42)).unwrap(), Some(42));
    }

    #[test]
    fn test_from_sql_nullable() {
        assert_eq!(i32::from_sql_nullable(SqlValue::Null).unwrap(), None);
        assert_eq!(i32::from_sql_nullable(SqlValue::I32(42)).unwrap(), Some(42));
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_from_sql_uuid() {
        let uuid = uuid::Uuid::new_v4();
        assert_eq!(uuid::Uuid::from_sql(SqlValue::Uuid(uuid)).unwrap(), uuid);
        assert_eq!(
            uuid::Uuid::from_sql(SqlValue::String(uuid.to_string())).unwrap(),
            uuid
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_from_sql_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(
            chrono::NaiveDate::from_sql(SqlValue::String("2024-01-15".into())).unwrap(),
            date
        );
    }
}
