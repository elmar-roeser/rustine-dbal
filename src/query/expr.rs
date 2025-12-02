//! Expression types for building WHERE clauses and conditions

use crate::core::SqlValue;

/// A SQL expression that can be used in WHERE clauses
#[derive(Debug, Clone)]
pub enum Expr {
    /// Column reference
    Column(String),
    /// Literal value
    Value(SqlValue),
    /// Parameter placeholder (e.g., $1, ?, :name)
    Param(String),
    /// Comparison: column op value
    Comparison(Box<Expr>, ComparisonOp, Box<Expr>),
    /// AND of multiple expressions
    And(Vec<Expr>),
    /// OR of multiple expressions
    Or(Vec<Expr>),
    /// NOT expression
    Not(Box<Expr>),
    /// IS NULL
    IsNull(Box<Expr>),
    /// IS NOT NULL
    IsNotNull(Box<Expr>),
    /// IN (values)
    In(Box<Expr>, Vec<Expr>),
    /// NOT IN (values)
    NotIn(Box<Expr>, Vec<Expr>),
    /// BETWEEN low AND high
    Between(Box<Expr>, Box<Expr>, Box<Expr>),
    /// LIKE pattern
    Like(Box<Expr>, String),
    /// Raw SQL expression
    Raw(String),
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    /// =
    Eq,
    /// <>
    Ne,
    /// <
    Lt,
    /// <=
    Le,
    /// >
    Gt,
    /// >=
    Ge,
}

impl ComparisonOp {
    /// Get the SQL representation
    #[must_use]
    pub const fn as_sql(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "<>",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }
}

impl Expr {
    /// Create a column reference
    #[must_use]
    pub fn col(name: impl Into<String>) -> Self {
        Self::Column(name.into())
    }

    /// Create a literal value
    #[must_use]
    pub fn val(value: impl Into<SqlValue>) -> Self {
        Self::Value(value.into())
    }

    /// Create a parameter placeholder
    #[must_use]
    pub fn param(name: impl Into<String>) -> Self {
        Self::Param(name.into())
    }

    /// Create a raw SQL expression
    #[must_use]
    pub fn raw(sql: impl Into<String>) -> Self {
        Self::Raw(sql.into())
    }

    /// Create an equality comparison: self = other
    #[must_use]
    pub fn eq(self, other: impl Into<Self>) -> Self {
        Self::Comparison(Box::new(self), ComparisonOp::Eq, Box::new(other.into()))
    }

    /// Create a not-equal comparison: self <> other
    #[must_use]
    pub fn ne(self, other: impl Into<Self>) -> Self {
        Self::Comparison(Box::new(self), ComparisonOp::Ne, Box::new(other.into()))
    }

    /// Create a less-than comparison: self < other
    #[must_use]
    pub fn lt(self, other: impl Into<Self>) -> Self {
        Self::Comparison(Box::new(self), ComparisonOp::Lt, Box::new(other.into()))
    }

    /// Create a less-than-or-equal comparison: self <= other
    #[must_use]
    pub fn le(self, other: impl Into<Self>) -> Self {
        Self::Comparison(Box::new(self), ComparisonOp::Le, Box::new(other.into()))
    }

    /// Create a greater-than comparison: self > other
    #[must_use]
    pub fn gt(self, other: impl Into<Self>) -> Self {
        Self::Comparison(Box::new(self), ComparisonOp::Gt, Box::new(other.into()))
    }

    /// Create a greater-than-or-equal comparison: self >= other
    #[must_use]
    pub fn ge(self, other: impl Into<Self>) -> Self {
        Self::Comparison(Box::new(self), ComparisonOp::Ge, Box::new(other.into()))
    }

    /// Create IS NULL expression
    #[must_use]
    pub fn is_null(self) -> Self {
        Self::IsNull(Box::new(self))
    }

    /// Create IS NOT NULL expression
    #[must_use]
    pub fn is_not_null(self) -> Self {
        Self::IsNotNull(Box::new(self))
    }

    /// Create IN expression
    #[must_use]
    pub fn in_list(self, values: Vec<Self>) -> Self {
        Self::In(Box::new(self), values)
    }

    /// Create NOT IN expression
    #[must_use]
    pub fn not_in_list(self, values: Vec<Self>) -> Self {
        Self::NotIn(Box::new(self), values)
    }

    /// Create BETWEEN expression
    #[must_use]
    pub fn between(self, low: impl Into<Self>, high: impl Into<Self>) -> Self {
        Self::Between(Box::new(self), Box::new(low.into()), Box::new(high.into()))
    }

    /// Create LIKE expression
    #[must_use]
    pub fn like(self, pattern: impl Into<String>) -> Self {
        Self::Like(Box::new(self), pattern.into())
    }

    /// Negate this expression
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Combine with AND
    #[must_use]
    pub fn and(self, other: impl Into<Self>) -> Self {
        match self {
            Self::And(mut exprs) => {
                exprs.push(other.into());
                Self::And(exprs)
            }
            _ => Self::And(vec![self, other.into()]),
        }
    }

    /// Combine with OR
    #[must_use]
    pub fn or(self, other: impl Into<Self>) -> Self {
        match self {
            Self::Or(mut exprs) => {
                exprs.push(other.into());
                Self::Or(exprs)
            }
            _ => Self::Or(vec![self, other.into()]),
        }
    }
}

// Convenience conversions
impl From<&str> for Expr {
    fn from(s: &str) -> Self {
        Self::Column(s.to_string())
    }
}

impl From<String> for Expr {
    fn from(s: String) -> Self {
        Self::Column(s)
    }
}

impl From<i32> for Expr {
    fn from(v: i32) -> Self {
        Self::Value(SqlValue::I32(v))
    }
}

impl From<i64> for Expr {
    fn from(v: i64) -> Self {
        Self::Value(SqlValue::I64(v))
    }
}

impl From<bool> for Expr {
    fn from(v: bool) -> Self {
        Self::Value(SqlValue::Bool(v))
    }
}

impl From<SqlValue> for Expr {
    fn from(v: SqlValue) -> Self {
        Self::Value(v)
    }
}

/// Helper function to create a column expression
#[must_use]
pub fn col(name: impl Into<String>) -> Expr {
    Expr::col(name)
}

/// Helper function to create a value expression
#[must_use]
pub fn val(value: impl Into<SqlValue>) -> Expr {
    Expr::val(value)
}

/// Helper function to create a parameter expression
#[must_use]
pub fn param(name: impl Into<String>) -> Expr {
    Expr::param(name)
}

/// Helper function for AND expressions
#[must_use]
pub const fn and(exprs: Vec<Expr>) -> Expr {
    Expr::And(exprs)
}

/// Helper function for OR expressions
#[must_use]
pub const fn or(exprs: Vec<Expr>) -> Expr {
    Expr::Or(exprs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_expr() {
        let expr = Expr::col("name");
        assert!(matches!(expr, Expr::Column(ref s) if s == "name"));
    }

    #[test]
    fn test_comparison() {
        let expr = Expr::col("age").gt(Expr::val(18i32));
        assert!(matches!(expr, Expr::Comparison(_, ComparisonOp::Gt, _)));
    }

    #[test]
    fn test_and_chain() {
        let expr = Expr::col("a").eq(1i32).and(Expr::col("b").eq(2i32));
        if let Expr::And(exprs) = expr {
            assert_eq!(exprs.len(), 2);
        } else {
            panic!("Expected And expression");
        }
    }

    #[test]
    fn test_or_chain() {
        let expr = Expr::col("status")
            .eq(Expr::val("active"))
            .or(Expr::col("status").eq(Expr::val("pending")));
        assert!(matches!(expr, Expr::Or(_)));
    }

    #[test]
    fn test_is_null() {
        let expr = Expr::col("deleted_at").is_null();
        assert!(matches!(expr, Expr::IsNull(_)));
    }

    #[test]
    fn test_between() {
        let expr = Expr::col("age").between(18i32, 65i32);
        assert!(matches!(expr, Expr::Between(_, _, _)));
    }

    #[test]
    fn test_like() {
        let expr = Expr::col("name").like("%test%");
        assert!(matches!(expr, Expr::Like(_, _)));
    }

    #[test]
    fn test_in_list() {
        let expr = Expr::col("status").in_list(vec![
            Expr::val("active"),
            Expr::val("pending"),
        ]);
        assert!(matches!(expr, Expr::In(_, _)));
    }

    #[test]
    fn test_comparison_op_sql() {
        assert_eq!(ComparisonOp::Eq.as_sql(), "=");
        assert_eq!(ComparisonOp::Ne.as_sql(), "<>");
        assert_eq!(ComparisonOp::Lt.as_sql(), "<");
        assert_eq!(ComparisonOp::Le.as_sql(), "<=");
        assert_eq!(ComparisonOp::Gt.as_sql(), ">");
        assert_eq!(ComparisonOp::Ge.as_sql(), ">=");
    }
}
