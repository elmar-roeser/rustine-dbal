//! # Rustine Core
//!
//! Core types, traits, and errors for the Rustine Database Abstraction Layer.
//!
//! This crate provides the foundational building blocks used across all Rustine crates:
//!
//! - **Error types**: Structured error hierarchy for all database operations
//! - **SqlValue**: Type-safe representation of database values
//! - **Type traits**: `ToSql` and `FromSql` for bidirectional type conversion
//! - **Configuration**: Connection and runtime configuration
//! - **ParameterType**: Parameter binding type information

mod error;
mod parameter;
mod sql_value;
mod to_sql;
mod from_sql;
mod config;

pub use error::*;
pub use parameter::*;
pub use sql_value::*;
pub use to_sql::*;
pub use from_sql::*;
pub use config::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{Error, Result, ConnectionError, TransactionError, SchemaError, QueryError};
    pub use crate::parameter::ParameterType;
    pub use crate::sql_value::SqlValue;
    pub use crate::to_sql::ToSql;
    pub use crate::from_sql::FromSql;
    pub use crate::config::{Configuration, ConnectionParams};
}
