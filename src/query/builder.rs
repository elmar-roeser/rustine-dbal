//! Query Builder for constructing SQL queries

use crate::core::SqlValue;
use crate::platform::Platform;
use super::expr::Expr;

/// The type of SQL query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
}

/// JOIN type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

impl JoinType {
    fn as_sql(&self) -> &'static str {
        match self {
            Self::Inner => "INNER JOIN",
            Self::Left => "LEFT JOIN",
            Self::Right => "RIGHT JOIN",
            Self::Full => "FULL JOIN",
            Self::Cross => "CROSS JOIN",
        }
    }
}

/// ORDER BY direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}

impl OrderDirection {
    fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// A JOIN clause
#[derive(Debug, Clone)]
struct Join {
    join_type: JoinType,
    table: String,
    alias: Option<String>,
    condition: Expr,
}

/// An ORDER BY clause
#[derive(Debug, Clone)]
struct OrderBy {
    column: String,
    direction: OrderDirection,
}

/// A fluent SQL query builder
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    query_type: QueryType,
    table: String,
    table_alias: Option<String>,
    columns: Vec<String>,
    values: Vec<Vec<SqlValue>>,
    set_values: Vec<(String, SqlValue)>,
    where_expr: Option<Expr>,
    joins: Vec<Join>,
    group_by: Vec<String>,
    having: Option<Expr>,
    order_by: Vec<OrderBy>,
    limit: Option<u64>,
    offset: Option<u64>,
    distinct: bool,
    returning: Vec<String>,
}

impl QueryBuilder {
    /// Create a new SELECT query builder
    pub fn select() -> Self {
        Self {
            query_type: QueryType::Select,
            table: String::new(),
            table_alias: None,
            columns: Vec::new(),
            values: Vec::new(),
            set_values: Vec::new(),
            where_expr: None,
            joins: Vec::new(),
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            distinct: false,
            returning: Vec::new(),
        }
    }

    /// Create a new INSERT query builder
    pub fn insert() -> Self {
        Self {
            query_type: QueryType::Insert,
            ..Self::select()
        }
    }

    /// Create a new UPDATE query builder
    pub fn update() -> Self {
        Self {
            query_type: QueryType::Update,
            ..Self::select()
        }
    }

    /// Create a new DELETE query builder
    pub fn delete() -> Self {
        Self {
            query_type: QueryType::Delete,
            ..Self::select()
        }
    }

    // ========================================================================
    // SELECT specific methods
    // ========================================================================

    /// Add columns to select
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns.extend(columns.iter().map(|s| s.to_string()));
        self
    }

    /// Add a single column to select
    pub fn column(mut self, column: &str) -> Self {
        self.columns.push(column.to_string());
        self
    }

    /// Select all columns (*)
    pub fn all(mut self) -> Self {
        self.columns.push("*".to_string());
        self
    }

    /// Make the SELECT DISTINCT
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    // ========================================================================
    // Common methods
    // ========================================================================

    /// Set the table to query from
    pub fn from(mut self, table: &str) -> Self {
        self.table = table.to_string();
        self
    }

    /// Set table alias
    pub fn alias(mut self, alias: &str) -> Self {
        self.table_alias = Some(alias.to_string());
        self
    }

    /// Set the table for INSERT
    pub fn into(mut self, table: &str) -> Self {
        self.table = table.to_string();
        self
    }

    /// Set the table for UPDATE
    pub fn table(mut self, table: &str) -> Self {
        self.table = table.to_string();
        self
    }

    // ========================================================================
    // WHERE clause
    // ========================================================================

    /// Add a WHERE condition
    pub fn where_expr(mut self, expr: Expr) -> Self {
        self.where_expr = Some(match self.where_expr {
            Some(existing) => existing.and(expr),
            None => expr,
        });
        self
    }

    /// Add a WHERE column = value condition
    pub fn where_eq(self, column: &str, value: impl Into<SqlValue>) -> Self {
        self.where_expr(Expr::col(column).eq(Expr::val(value.into())))
    }

    /// Add a WHERE column IS NULL condition
    pub fn where_null(self, column: &str) -> Self {
        self.where_expr(Expr::col(column).is_null())
    }

    /// Add a WHERE column IS NOT NULL condition
    pub fn where_not_null(self, column: &str) -> Self {
        self.where_expr(Expr::col(column).is_not_null())
    }

    /// Add a WHERE column IN (values) condition
    pub fn where_in(self, column: &str, values: Vec<SqlValue>) -> Self {
        let exprs: Vec<Expr> = values.into_iter().map(Expr::val).collect();
        self.where_expr(Expr::col(column).in_list(exprs))
    }

    /// Add a WHERE column LIKE pattern condition
    pub fn where_like(self, column: &str, pattern: &str) -> Self {
        self.where_expr(Expr::col(column).like(pattern))
    }

    /// Add a WHERE with OR condition
    pub fn or_where(mut self, expr: Expr) -> Self {
        self.where_expr = Some(match self.where_expr {
            Some(existing) => existing.or(expr),
            None => expr,
        });
        self
    }

    // ========================================================================
    // JOIN clauses
    // ========================================================================

    /// Add an INNER JOIN
    pub fn inner_join(mut self, table: &str, condition: Expr) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            table: table.to_string(),
            alias: None,
            condition,
        });
        self
    }

    /// Add a LEFT JOIN
    pub fn left_join(mut self, table: &str, condition: Expr) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Left,
            table: table.to_string(),
            alias: None,
            condition,
        });
        self
    }

    /// Add a RIGHT JOIN
    pub fn right_join(mut self, table: &str, condition: Expr) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Right,
            table: table.to_string(),
            alias: None,
            condition,
        });
        self
    }

    /// Add a JOIN with alias
    pub fn join_alias(mut self, join_type: JoinType, table: &str, alias: &str, condition: Expr) -> Self {
        self.joins.push(Join {
            join_type,
            table: table.to_string(),
            alias: Some(alias.to_string()),
            condition,
        });
        self
    }

    // ========================================================================
    // GROUP BY and HAVING
    // ========================================================================

    /// Add GROUP BY columns
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by.extend(columns.iter().map(|s| s.to_string()));
        self
    }

    /// Add HAVING condition
    pub fn having(mut self, expr: Expr) -> Self {
        self.having = Some(expr);
        self
    }

    // ========================================================================
    // ORDER BY, LIMIT, OFFSET
    // ========================================================================

    /// Add ORDER BY clause
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order_by.push(OrderBy {
            column: column.to_string(),
            direction,
        });
        self
    }

    /// Add ORDER BY ASC
    pub fn order_by_asc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Asc)
    }

    /// Add ORDER BY DESC
    pub fn order_by_desc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Desc)
    }

    /// Set LIMIT
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set OFFSET
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    // ========================================================================
    // INSERT specific methods
    // ========================================================================

    /// Set columns for INSERT
    pub fn insert_columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a row of values for INSERT
    pub fn values(mut self, values: Vec<SqlValue>) -> Self {
        self.values.push(values);
        self
    }

    /// Add multiple rows for INSERT
    pub fn values_batch(mut self, rows: Vec<Vec<SqlValue>>) -> Self {
        self.values.extend(rows);
        self
    }

    // ========================================================================
    // UPDATE specific methods
    // ========================================================================

    /// Set a column value for UPDATE
    pub fn set(mut self, column: &str, value: impl Into<SqlValue>) -> Self {
        self.set_values.push((column.to_string(), value.into()));
        self
    }

    // ========================================================================
    // RETURNING clause
    // ========================================================================

    /// Add RETURNING clause (PostgreSQL, SQLite 3.35+)
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.returning.extend(columns.iter().map(|s| s.to_string()));
        self
    }

    // ========================================================================
    // SQL Generation
    // ========================================================================

    /// Build the SQL query for a specific platform
    pub fn to_sql<P: Platform>(&self, platform: &P) -> String {
        match self.query_type {
            QueryType::Select => self.build_select(platform),
            QueryType::Insert => self.build_insert(platform),
            QueryType::Update => self.build_update(platform),
            QueryType::Delete => self.build_delete(platform),
        }
    }

    fn build_select<P: Platform>(&self, platform: &P) -> String {
        let mut sql = String::from("SELECT ");

        if self.distinct {
            sql.push_str("DISTINCT ");
        }

        // Columns
        if self.columns.is_empty() {
            sql.push('*');
        } else {
            let cols: Vec<String> = self.columns.iter()
                .map(|c| if c == "*" { c.clone() } else { platform.quote_identifier(c) })
                .collect();
            sql.push_str(&cols.join(", "));
        }

        // FROM
        sql.push_str(" FROM ");
        sql.push_str(&platform.quote_identifier(&self.table));
        if let Some(ref alias) = self.table_alias {
            sql.push_str(" AS ");
            sql.push_str(&platform.quote_identifier(alias));
        }

        // JOINs
        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join.join_type.as_sql());
            sql.push(' ');
            sql.push_str(&platform.quote_identifier(&join.table));
            if let Some(ref alias) = join.alias {
                sql.push_str(" AS ");
                sql.push_str(&platform.quote_identifier(alias));
            }
            sql.push_str(" ON ");
            sql.push_str(&self.expr_to_sql(&join.condition, platform));
        }

        // WHERE
        if let Some(ref where_expr) = self.where_expr {
            sql.push_str(" WHERE ");
            sql.push_str(&self.expr_to_sql(where_expr, platform));
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            let cols: Vec<String> = self.group_by.iter()
                .map(|c| platform.quote_identifier(c))
                .collect();
            sql.push_str(&cols.join(", "));
        }

        // HAVING
        if let Some(ref having) = self.having {
            sql.push_str(" HAVING ");
            sql.push_str(&self.expr_to_sql(having, platform));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let orders: Vec<String> = self.order_by.iter()
                .map(|o| format!("{} {}", platform.quote_identifier(&o.column), o.direction.as_sql()))
                .collect();
            sql.push_str(&orders.join(", "));
        }

        // LIMIT/OFFSET
        sql.push_str(&platform.limit_offset_sql(self.limit, self.offset));

        sql
    }

    fn build_insert<P: Platform>(&self, platform: &P) -> String {
        let mut sql = String::from("INSERT INTO ");
        sql.push_str(&platform.quote_identifier(&self.table));

        // Columns
        if !self.columns.is_empty() {
            sql.push_str(" (");
            let cols: Vec<String> = self.columns.iter()
                .map(|c| platform.quote_identifier(c))
                .collect();
            sql.push_str(&cols.join(", "));
            sql.push(')');
        }

        // VALUES
        sql.push_str(" VALUES ");
        let rows: Vec<String> = self.values.iter()
            .map(|row| {
                let vals: Vec<String> = row.iter()
                    .map(|v| self.value_to_sql(v))
                    .collect();
                format!("({})", vals.join(", "))
            })
            .collect();
        sql.push_str(&rows.join(", "));

        // RETURNING
        if !self.returning.is_empty() && platform.supports_returning() {
            sql.push_str(" RETURNING ");
            let cols: Vec<String> = self.returning.iter()
                .map(|c| platform.quote_identifier(c))
                .collect();
            sql.push_str(&cols.join(", "));
        }

        sql
    }

    fn build_update<P: Platform>(&self, platform: &P) -> String {
        let mut sql = String::from("UPDATE ");
        sql.push_str(&platform.quote_identifier(&self.table));

        // SET
        sql.push_str(" SET ");
        let sets: Vec<String> = self.set_values.iter()
            .map(|(col, val)| {
                format!("{} = {}", platform.quote_identifier(col), self.value_to_sql(val))
            })
            .collect();
        sql.push_str(&sets.join(", "));

        // WHERE
        if let Some(ref where_expr) = self.where_expr {
            sql.push_str(" WHERE ");
            sql.push_str(&self.expr_to_sql(where_expr, platform));
        }

        // RETURNING
        if !self.returning.is_empty() && platform.supports_returning() {
            sql.push_str(" RETURNING ");
            let cols: Vec<String> = self.returning.iter()
                .map(|c| platform.quote_identifier(c))
                .collect();
            sql.push_str(&cols.join(", "));
        }

        sql
    }

    fn build_delete<P: Platform>(&self, platform: &P) -> String {
        let mut sql = String::from("DELETE FROM ");
        sql.push_str(&platform.quote_identifier(&self.table));

        // WHERE
        if let Some(ref where_expr) = self.where_expr {
            sql.push_str(" WHERE ");
            sql.push_str(&self.expr_to_sql(where_expr, platform));
        }

        // RETURNING
        if !self.returning.is_empty() && platform.supports_returning() {
            sql.push_str(" RETURNING ");
            let cols: Vec<String> = self.returning.iter()
                .map(|c| platform.quote_identifier(c))
                .collect();
            sql.push_str(&cols.join(", "));
        }

        sql
    }

    fn expr_to_sql<P: Platform>(&self, expr: &Expr, platform: &P) -> String {
        match expr {
            Expr::Column(name) => platform.quote_identifier(name),
            Expr::Value(val) => self.value_to_sql(val),
            Expr::Param(name) => name.clone(),
            Expr::Comparison(left, op, right) => {
                format!(
                    "{} {} {}",
                    self.expr_to_sql(left, platform),
                    op.as_sql(),
                    self.expr_to_sql(right, platform)
                )
            }
            Expr::And(exprs) => {
                let parts: Vec<String> = exprs.iter()
                    .map(|e| self.expr_to_sql(e, platform))
                    .collect();
                format!("({})", parts.join(" AND "))
            }
            Expr::Or(exprs) => {
                let parts: Vec<String> = exprs.iter()
                    .map(|e| self.expr_to_sql(e, platform))
                    .collect();
                format!("({})", parts.join(" OR "))
            }
            Expr::Not(inner) => {
                format!("NOT ({})", self.expr_to_sql(inner, platform))
            }
            Expr::IsNull(inner) => {
                format!("{} IS NULL", self.expr_to_sql(inner, platform))
            }
            Expr::IsNotNull(inner) => {
                format!("{} IS NOT NULL", self.expr_to_sql(inner, platform))
            }
            Expr::In(col, values) => {
                let vals: Vec<String> = values.iter()
                    .map(|v| self.expr_to_sql(v, platform))
                    .collect();
                format!("{} IN ({})", self.expr_to_sql(col, platform), vals.join(", "))
            }
            Expr::NotIn(col, values) => {
                let vals: Vec<String> = values.iter()
                    .map(|v| self.expr_to_sql(v, platform))
                    .collect();
                format!("{} NOT IN ({})", self.expr_to_sql(col, platform), vals.join(", "))
            }
            Expr::Between(col, low, high) => {
                format!(
                    "{} BETWEEN {} AND {}",
                    self.expr_to_sql(col, platform),
                    self.expr_to_sql(low, platform),
                    self.expr_to_sql(high, platform)
                )
            }
            Expr::Like(col, pattern) => {
                format!("{} LIKE {}", self.expr_to_sql(col, platform), platform.quote_string(pattern))
            }
            Expr::Raw(sql) => sql.clone(),
        }
    }

    fn value_to_sql(&self, value: &SqlValue) -> String {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{PostgresPlatform, MySqlPlatform, SqlitePlatform};

    #[test]
    fn test_simple_select() {
        let sql = QueryBuilder::select()
            .columns(&["id", "name", "email"])
            .from("users")
            .to_sql(&PostgresPlatform);

        assert_eq!(sql, "SELECT \"id\", \"name\", \"email\" FROM \"users\"");
    }

    #[test]
    fn test_select_all() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .to_sql(&PostgresPlatform);

        assert_eq!(sql, "SELECT * FROM \"users\"");
    }

    #[test]
    fn test_select_distinct() {
        let sql = QueryBuilder::select()
            .distinct()
            .column("status")
            .from("orders")
            .to_sql(&PostgresPlatform);

        assert_eq!(sql, "SELECT DISTINCT \"status\" FROM \"orders\"");
    }

    #[test]
    fn test_select_with_where() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .where_eq("id", 1i64)
            .to_sql(&PostgresPlatform);

        assert_eq!(sql, "SELECT * FROM \"users\" WHERE \"id\" = 1");
    }

    #[test]
    fn test_select_with_multiple_where() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .where_eq("active", true)
            .where_eq("role", SqlValue::String("admin".to_string()))
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("WHERE"));
        assert!(sql.contains("AND"));
    }

    #[test]
    fn test_select_with_join() {
        let sql = QueryBuilder::select()
            .columns(&["u.name", "o.total"])
            .from("users")
            .alias("u")
            .inner_join("orders", Expr::col("u.id").eq(Expr::col("o.user_id")))
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("INNER JOIN"));
        assert!(sql.contains("ON"));
    }

    #[test]
    fn test_select_with_order_and_limit() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .order_by_desc("created_at")
            .limit(10)
            .offset(20)
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("ORDER BY \"created_at\" DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 20"));
    }

    #[test]
    fn test_select_with_group_by() {
        let sql = QueryBuilder::select()
            .column("status")
            .column("COUNT(*)")
            .from("orders")
            .group_by(&["status"])
            .having(Expr::raw("COUNT(*) > 5"))
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("GROUP BY \"status\""));
        assert!(sql.contains("HAVING COUNT(*) > 5"));
    }

    #[test]
    fn test_insert() {
        let sql = QueryBuilder::insert()
            .into("users")
            .insert_columns(&["name", "email"])
            .values(vec![
                SqlValue::String("Alice".to_string()),
                SqlValue::String("alice@example.com".to_string()),
            ])
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("INSERT INTO \"users\""));
        assert!(sql.contains("(\"name\", \"email\")"));
        assert!(sql.contains("VALUES ('Alice', 'alice@example.com')"));
    }

    #[test]
    fn test_insert_multiple_rows() {
        let sql = QueryBuilder::insert()
            .into("users")
            .insert_columns(&["name"])
            .values(vec![SqlValue::String("Alice".to_string())])
            .values(vec![SqlValue::String("Bob".to_string())])
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("('Alice'), ('Bob')"));
    }

    #[test]
    fn test_insert_with_returning() {
        let sql = QueryBuilder::insert()
            .into("users")
            .insert_columns(&["name"])
            .values(vec![SqlValue::String("Alice".to_string())])
            .returning(&["id"])
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("RETURNING \"id\""));
    }

    #[test]
    fn test_update() {
        let sql = QueryBuilder::update()
            .table("users")
            .set("name", SqlValue::String("Bob".to_string()))
            .set("active", true)
            .where_eq("id", 1i64)
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("UPDATE \"users\""));
        assert!(sql.contains("SET \"name\" = 'Bob', \"active\" = true"));
        assert!(sql.contains("WHERE \"id\" = 1"));
    }

    #[test]
    fn test_delete() {
        let sql = QueryBuilder::delete()
            .from("users")
            .where_eq("id", 1i64)
            .to_sql(&PostgresPlatform);

        assert_eq!(sql, "DELETE FROM \"users\" WHERE \"id\" = 1");
    }

    #[test]
    fn test_mysql_quoting() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .to_sql(&MySqlPlatform);

        assert_eq!(sql, "SELECT * FROM `users`");
    }

    #[test]
    fn test_sqlite_quoting() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .to_sql(&SqlitePlatform);

        assert_eq!(sql, "SELECT * FROM \"users\"");
    }

    #[test]
    fn test_where_in() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .where_in("status", vec![
                SqlValue::String("active".to_string()),
                SqlValue::String("pending".to_string()),
            ])
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("\"status\" IN ('active', 'pending')"));
    }

    #[test]
    fn test_where_like() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .where_like("name", "%test%")
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("\"name\" LIKE '%test%'"));
    }

    #[test]
    fn test_where_null() {
        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .where_null("deleted_at")
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("\"deleted_at\" IS NULL"));
    }

    #[test]
    fn test_complex_where() {
        let expr = Expr::col("age").ge(18i32)
            .and(Expr::col("age").le(65i32))
            .and(Expr::col("status").eq(Expr::val("active")));

        let sql = QueryBuilder::select()
            .all()
            .from("users")
            .where_expr(expr)
            .to_sql(&PostgresPlatform);

        assert!(sql.contains("\"age\" >= 18"));
        assert!(sql.contains("\"age\" <= 65"));
        assert!(sql.contains("\"status\" = 'active'"));
    }
}
