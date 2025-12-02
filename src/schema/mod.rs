//! # Schema Module
//!
//! Schema introspection and management for database operations.
//!
//! This module provides types for representing database schema objects
//! (tables, columns, indexes, foreign keys) and the `SchemaManager`
//! for introspecting and manipulating schemas.
//!
//! ## Example
//!
//! ```rust,ignore
//! use rustine_dbal::schema::SchemaManager;
//! use rustine_dbal::platform::SqlitePlatform;
//!
//! // Create a schema manager
//! let manager = SchemaManager::new(&connection, &SqlitePlatform);
//!
//! // List all tables
//! let tables = manager.list_table_names().await?;
//!
//! // Introspect a table
//! let table_info = manager.introspect_table("users").await?;
//! for column in &table_info.columns {
//!     println!("{}: {} (nullable: {})", column.name, column.type_name, column.nullable);
//! }
//!
//! // Check if table exists
//! if manager.table_exists("users").await? {
//!     println!("users table exists!");
//! }
//! ```

mod manager;

pub use manager::{SchemaManager, ColumnInfo, IndexInfo, ForeignKeyInfo, TableInfo};
