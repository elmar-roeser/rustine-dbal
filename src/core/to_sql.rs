//! ToSql trait for converting Rust types to SQL values
//!
//! This trait enables any Rust type to be converted into a [`SqlValue`]
//! for use in query parameters.

use super::{Result, SqlValue};

/// Trait for types that can be converted to SQL values
///
/// Implement this trait for custom types that need to be used as query parameters.
///
/// # Example
///
/// ```rust
/// use rustine_dbal::{Result, SqlValue};
/// use rustine_dbal::core::ToSql;
///
/// struct Money {
///     cents: i64,
/// }
///
/// impl ToSql for Money {
///     fn to_sql(&self) -> Result<SqlValue> {
///         Ok(SqlValue::I64(self.cents))
///     }
/// }
/// ```
pub trait ToSql {
    /// Convert this value to a SQL value
    fn to_sql(&self) -> Result<SqlValue>;
}

// Implement for all types that have Into<SqlValue>
impl ToSql for bool {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Bool(*self))
    }
}

impl ToSql for i8 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::I8(*self))
    }
}

impl ToSql for i16 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::I16(*self))
    }
}

impl ToSql for i32 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::I32(*self))
    }
}

impl ToSql for i64 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::I64(*self))
    }
}

impl ToSql for u32 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::U32(*self))
    }
}

impl ToSql for u64 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::U64(*self))
    }
}

impl ToSql for f32 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::F32(*self))
    }
}

impl ToSql for f64 {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::F64(*self))
    }
}

impl ToSql for String {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::String(self.clone()))
    }
}

impl ToSql for str {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::String(self.to_owned()))
    }
}

impl ToSql for &str {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::String((*self).to_owned()))
    }
}

impl ToSql for Vec<u8> {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Bytes(self.clone()))
    }
}

impl ToSql for [u8] {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Bytes(self.to_vec()))
    }
}

impl<T: ToSql> ToSql for Option<T> {
    fn to_sql(&self) -> Result<SqlValue> {
        match self {
            Some(v) => v.to_sql(),
            None => Ok(SqlValue::Null),
        }
    }
}

impl<T: ToSql> ToSql for &T {
    fn to_sql(&self) -> Result<SqlValue> {
        (*self).to_sql()
    }
}

impl ToSql for SqlValue {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(self.clone())
    }
}

// Feature-gated implementations
#[cfg(feature = "chrono")]
impl ToSql for chrono::NaiveDate {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Date(*self))
    }
}

#[cfg(feature = "chrono")]
impl ToSql for chrono::NaiveTime {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Time(*self))
    }
}

#[cfg(feature = "chrono")]
impl ToSql for chrono::NaiveDateTime {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::DateTime(*self))
    }
}

#[cfg(feature = "chrono")]
impl ToSql for chrono::DateTime<chrono::Utc> {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::DateTimeUtc(*self))
    }
}

#[cfg(feature = "uuid")]
impl ToSql for uuid::Uuid {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Uuid(*self))
    }
}

#[cfg(feature = "json")]
impl ToSql for serde_json::Value {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Json(self.clone()))
    }
}

#[cfg(feature = "decimal")]
impl ToSql for rust_decimal::Decimal {
    fn to_sql(&self) -> Result<SqlValue> {
        Ok(SqlValue::Decimal(*self))
    }
}

/// Extension trait for converting iterables of ToSql items to Vec<SqlValue>
pub trait ToSqlVec {
    /// Convert to a vector of SQL values
    fn to_sql_vec(&self) -> Result<Vec<SqlValue>>;
}

impl<T: ToSql> ToSqlVec for [T] {
    fn to_sql_vec(&self) -> Result<Vec<SqlValue>> {
        self.iter().map(|v| v.to_sql()).collect()
    }
}

impl<T: ToSql> ToSqlVec for Vec<T> {
    fn to_sql_vec(&self) -> Result<Vec<SqlValue>> {
        self.iter().map(|v| v.to_sql()).collect()
    }
}

/// Trait object safe version of ToSql
///
/// This allows storing heterogeneous parameter types in collections.
pub trait DynToSql: Send + Sync {
    /// Convert to a SQL value (boxed version for trait objects)
    fn to_sql_dyn(&self) -> Result<SqlValue>;
}

impl<T: ToSql + Send + Sync> DynToSql for T {
    fn to_sql_dyn(&self) -> Result<SqlValue> {
        self.to_sql()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_sql_primitives() {
        assert_eq!(true.to_sql().unwrap(), SqlValue::Bool(true));
        assert_eq!(42i32.to_sql().unwrap(), SqlValue::I32(42));
        assert_eq!(42i64.to_sql().unwrap(), SqlValue::I64(42));
        assert_eq!(3.14f64.to_sql().unwrap(), SqlValue::F64(3.14));
    }

    #[test]
    fn test_to_sql_string() {
        assert_eq!(
            "hello".to_sql().unwrap(),
            SqlValue::String("hello".to_string())
        );
        assert_eq!(
            String::from("world").to_sql().unwrap(),
            SqlValue::String("world".to_string())
        );
    }

    #[test]
    fn test_to_sql_option() {
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;

        assert_eq!(some_val.to_sql().unwrap(), SqlValue::I32(42));
        assert_eq!(none_val.to_sql().unwrap(), SqlValue::Null);
    }

    #[test]
    fn test_to_sql_reference() {
        let value = 42i32;
        let reference = &value;
        assert_eq!(reference.to_sql().unwrap(), SqlValue::I32(42));
    }

    #[test]
    fn test_to_sql_vec() {
        let values = vec![1i32, 2, 3];
        let sql_values = values.to_sql_vec().unwrap();
        assert_eq!(
            sql_values,
            vec![SqlValue::I32(1), SqlValue::I32(2), SqlValue::I32(3)]
        );
    }

    #[test]
    fn test_to_sql_slice() {
        let values: &[i32] = &[1, 2, 3];
        let sql_values = values.to_sql_vec().unwrap();
        assert_eq!(
            sql_values,
            vec![SqlValue::I32(1), SqlValue::I32(2), SqlValue::I32(3)]
        );
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_to_sql_uuid() {
        let uuid = uuid::Uuid::new_v4();
        assert_eq!(uuid.to_sql().unwrap(), SqlValue::Uuid(uuid));
    }
}
