# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2025-12-02

### Added
- **SQLite Driver** (Epic 2: Database Connectivity)
  - `SqliteDriver` - Driver implementation with sqlx
  - `SqliteConnection` - Connection management with true single connection (not pool)
  - `SqliteStatement` - Prepared statements with positional/named parameters
  - `SqliteResult` - Result set iteration
  - Feature flag `sqlite` for optional activation
  - Improved type detection for dynamic SQLite expressions (COUNT(*), etc.)

- **Transaction Management** (Epic 3)
  - `Connection<D>` - High-level connection wrapper over driver traits
  - Nested transactions via savepoints (`RUSTINE_1`, `RUSTINE_2`, etc.)
  - `begin_transaction()`, `commit()`, `rollback()` with nesting counter
  - `transactional_boxed()` - Transactional closure API with auto-commit/rollback
  - `in_transaction()` - Simple result-based commit/rollback
  - Transaction state: `is_transaction_active()`, `is_rollback_only()`, `set_rollback_only()`
  - `TransactionGuard` for RAII-style transaction management
  - Drop guard with warning for open transactions (tracing feature)
  - 10 new unit tests for transaction functionality (65 tests total)

### Changed
- **BREAKING**: Project restructured from multi-crate workspace to monolith crate
- Crate renamed from `rustine` to `rustine-dbal`
- Modules: `rustine-core` → `core/`, `rustine-driver` → `driver/`, etc.
- Imports change: `use rustine_dbal::core::*` instead of `use rustine_core::*`

## [0.1.0] - 2024-12-02

### Added

#### Core Foundation (Epic 1)
- **core**: Error hierarchy (`Error`, `ConnectionError`, `TransactionError`, `QueryError`, `SchemaError`)
- **core**: `SqlValue` enum with 15+ variants for all SQL types
- **core**: `ToSql` trait for Rust → SQL conversion
- **core**: `FromSql` trait for SQL → Rust conversion
- **core**: `ParameterType` enum for prepared statement binding
- **core**: `ConnectionParams` for connection configuration
- **core**: `Configuration` for runtime settings
- **core**: `IsolationLevel` enum for transaction isolation
- **core**: Feature flags for `chrono`, `uuid`, `json`, `decimal`

#### Driver Abstraction
- **driver**: `Driver` trait for database drivers
- **driver**: `DriverConnection` trait for connections
- **driver**: `DriverStatement` trait for prepared statements
- **driver**: `DriverResult` trait for query results

#### Platform Abstraction
- **platform**: `Platform` trait for SQL dialects
- **platform**: `PostgresPlatform` basic structure
- **platform**: `MySqlPlatform` basic structure
- **platform**: `SqlitePlatform` basic structure

#### Documentation
- PRD (Product Requirements Document) following BMAD Method v6
- Architecture Decision Document with 7 ADRs
- Epic breakdown with 6 epics and 29 stories
- Doctrine DBAL analysis documentation (8 documents)
- Conventional Commits guidelines
- SemVer documentation in Cargo.toml

#### Tests
- 43 unit tests (38 in core, 5 in platform)
- 3 doc tests

### Infrastructure
- Monolith crate structure (`rustine-dbal`)
- GitHub repository set up
- .gitignore for Rust projects

[Unreleased]: https://github.com/elmar-roeser/rustine-dbal/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/elmar-roeser/rustine-dbal/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/elmar-roeser/rustine-dbal/releases/tag/v0.1.0
