//! # Query Module
//!
//! Query builder for constructing SQL queries programmatically.
//!
//! This module provides a fluent API for building SELECT, INSERT, UPDATE,
//! and DELETE queries in a type-safe manner.
//!
//! ## Example
//!
//! ```rust
//! use rustine_dbal::query::{QueryBuilder, Expr};
//! use rustine_dbal::platform::PostgresPlatform;
//!
//! // SELECT query
//! let sql = QueryBuilder::select()
//!     .columns(&["id", "name", "email"])
//!     .from("users")
//!     .where_eq("active", true)
//!     .order_by_desc("created_at")
//!     .limit(10)
//!     .to_sql(&PostgresPlatform);
//!
//! // INSERT query
//! let sql = QueryBuilder::insert()
//!     .into("users")
//!     .insert_columns(&["name", "email"])
//!     .values(vec!["Alice".into(), "alice@example.com".into()])
//!     .returning(&["id"])
//!     .to_sql(&PostgresPlatform);
//!
//! // UPDATE query
//! use rustine_dbal::SqlValue;
//! let sql = QueryBuilder::update()
//!     .table("users")
//!     .set("name", SqlValue::String("Bob".to_string()))
//!     .where_eq("id", 1i64)
//!     .to_sql(&PostgresPlatform);
//!
//! // DELETE query
//! let sql = QueryBuilder::delete()
//!     .from("users")
//!     .where_eq("id", 1i64)
//!     .to_sql(&PostgresPlatform);
//! ```

mod builder;
mod expr;

pub use builder::{QueryBuilder, QueryType, JoinType, OrderDirection};
pub use expr::{Expr, ComparisonOp, col, val, param, and, or};
