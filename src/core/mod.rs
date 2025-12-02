//! # Core Module
//!
//! Core types, traits, and errors for the Rustine Database Abstraction Layer.
//!
//! This module provides the foundational building blocks:
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
