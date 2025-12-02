# Architecture Decision Document - Rustine

**Author:** Elmar
**Date:** 2024-12-02
**Version:** 1.0
**Based on:** BMAD Method v6

---

## Project Context

### Technology Stack

| Layer | Technology | Version | Rationale |
|-------|------------|---------|-----------|
| **Language** | Rust | 2021 Edition | Zielsprache der Portierung |
| **Async Runtime** | Tokio | 1.x | De-facto Standard, beste Ökosystem-Integration |
| **DB Client Base** | sqlx | 0.8.x | Compile-time checked queries, async-native |
| **Error Handling** | thiserror | 2.x | Ergonomische Error-Derivation |
| **Serialization** | serde | 1.x | Standard für Rust-Serialization |
| **DateTime** | chrono | 0.4.x | Umfassende DateTime-Unterstützung |
| **Decimal** | rust_decimal | 1.x | Präzise Dezimalzahlen |
| **UUID** | uuid | 1.x | UUID-Unterstützung |
| **JSON** | serde_json | 1.x | JSON-Verarbeitung |
| **Tracing** | tracing | 0.1.x | Structured Logging (optional) |

### Constraints

| Constraint | Beschreibung |
|------------|--------------|
| **No Unsafe** | Kein `unsafe` Code in Public API |
| **MSRV** | Minimum Supported Rust Version: 1.75 (für async traits) |
| **No Panics** | Public API darf nicht panicken |
| **Send + Sync** | Alle Public Types wo sinnvoll |
| **Zero-Cost** | Abstractions ohne Runtime-Overhead wo möglich |

### Input Documents

- [PRD](prd.md) - 49 Functional Requirements
- [Doctrine DBAL Analyse](00-overview.md) - Referenz-Architektur
- [Rust Mapping](07-rust-mapping.md) - Portierungs-Strategie

---

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- ADR-001: Crate-Struktur
- ADR-002: Async-Strategie
- ADR-003: Error-Handling
- ADR-004: Trait-Design für Driver/Platform

**Important Decisions (Shape Architecture):**
- ADR-005: Connection State Management
- ADR-006: Type System Design
- ADR-007: QueryBuilder Ownership

**Deferred Decisions (Post-MVP):**
- Connection Pooling Strategie (nutzt sqlx built-in)
- Caching Layer
- Middleware/Hook System

---

### ADR-001: Crate-Struktur

**Status:** Accepted

**Context:**
Rustine ist ein komplexes System mit mehreren Komponenten. Die Struktur muss:
- Modulare Nutzung ermöglichen (nur QueryBuilder, ohne Schema)
- Klare Abhängigkeiten definieren
- Feature-Flags für optionale Funktionen unterstützen

**Decision:**
Multi-Crate Workspace mit folgender Struktur:

```
rustine/                    # Workspace Root
├── Cargo.toml              # Workspace Definition
├── rustine/                # Meta-Crate (Re-exports)
├── rustine-core/           # Basis-Traits, Types, Errors
├── rustine-driver/         # Driver-Trait + Implementierungen
├── rustine-platform/       # Platform-Trait + SQL-Dialekte
├── rustine-query/          # QueryBuilder
├── rustine-schema/         # Schema-Introspection
└── rustine-derive/         # Proc-Macros (später)
```

**Consequences:**
- Nutzer können `rustine` als All-in-One oder einzelne Crates nutzen
- Klare Dependency-Hierarchie verhindert zirkuläre Abhängigkeiten
- Compile-Zeiten sind optimiert durch separate Compilation Units

---

### ADR-002: Async-Strategie

**Status:** Accepted

**Context:**
Datenbankoperationen sind I/O-bound. Rust bietet async/await, aber erfordert Runtime-Wahl.

**Decision:**
- **Async-first Design** - Alle DB-Operationen sind async
- **Tokio als primäre Runtime** - via Feature-Flag
- **Runtime-agnostisch wo möglich** - Abstraktion über async-trait
- **Kein Sync-Wrapper** - Sync-Nutzer können `block_on` selbst aufrufen

**API-Design:**

```rust
impl Connection {
    pub async fn execute_query(&self, sql: &str, params: &[&dyn ToSql])
        -> Result<impl Stream<Item = Result<Row, Error>>, Error>;

    pub async fn execute_statement(&self, sql: &str, params: &[&dyn ToSql])
        -> Result<u64, Error>;
}
```

**Consequences:**
- Maximale Performance für concurrent Workloads
- Nutzer benötigen Tokio Runtime (oder kompatible)
- `async_trait` Crate für trait-Methoden bis Rust stabilisiert

---

### ADR-003: Error-Handling Strategie

**Status:** Accepted

**Context:**
Doctrine DBAL hat eine Exception-Hierarchie. Rust braucht ein Error-Enum Design.

**Decision:**
Hierarchisches Error-Enum mit `thiserror`:

```rust
// rustine-core/src/error.rs

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Query error: {message}")]
    Query { message: String, sql: Option<String> },

    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),

    #[error("Type conversion error: {0}")]
    Conversion(#[from] ConversionError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),

    #[error(transparent)]
    Driver(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection refused: {0}")]
    Refused(String),

    #[error("Connection lost")]
    Lost,

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Database not found: {0}")]
    DatabaseNotFound(String),
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("No active transaction")]
    NoActiveTransaction,

    #[error("Transaction marked rollback-only")]
    RollbackOnly,

    #[error("Savepoints not supported by this platform")]
    SavepointsNotSupported,
}

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Column not found: {table}.{column}")]
    ColumnNotFound { table: String, column: String },
}
```

**Consequences:**
- Pattern-Matching auf Error-Arten möglich
- Kontext-Information verfügbar
- Keine Panik-Gefahr in Public API

---

### ADR-004: Trait-Design für Driver und Platform

**Status:** Accepted

**Context:**
Doctrine DBAL nutzt PHP-Interfaces und Abstract-Classes. Rust braucht Traits.

**Decision:**

**Driver Trait Hierarchie:**

```rust
// rustine-driver/src/lib.rs

/// Haupt-Driver-Trait für Datenbank-Verbindungen
#[async_trait]
pub trait Driver: Send + Sync {
    type Connection: DriverConnection;

    /// Stellt neue Verbindung her
    async fn connect(&self, params: &ConnectionParams) -> Result<Self::Connection, Error>;

    /// Gibt passende Platform zurück
    fn platform(&self, server_version: &str) -> Arc<dyn Platform>;
}

/// Low-Level Connection zum Datenbank-Server
#[async_trait]
pub trait DriverConnection: Send {
    type Statement<'conn>: DriverStatement<'conn> where Self: 'conn;
    type TransactionGuard<'conn>: TransactionGuard<'conn> where Self: 'conn;

    async fn prepare(&self, sql: &str) -> Result<Self::Statement<'_>, Error>;
    async fn execute(&self, sql: &str) -> Result<u64, Error>;

    async fn begin(&mut self) -> Result<Self::TransactionGuard<'_>, Error>;

    fn server_version(&self) -> &str;
}

/// Prepared Statement
#[async_trait]
pub trait DriverStatement<'conn>: Send {
    async fn execute(&mut self, params: &[SqlValue]) -> Result<DriverResult, Error>;
}
```

**Platform Trait:**

```rust
// rustine-platform/src/lib.rs

pub trait Platform: Send + Sync {
    // === Identification ===
    fn name(&self) -> &'static str;

    // === Quoting ===
    fn quote_identifier(&self, id: &str) -> String;
    fn quote_string_literal(&self, value: &str) -> String;

    // === Type Declarations ===
    fn integer_type_sql(&self) -> &'static str;
    fn bigint_type_sql(&self) -> &'static str;
    fn string_type_sql(&self, length: Option<u32>) -> String;
    fn text_type_sql(&self) -> &'static str;
    fn boolean_type_sql(&self) -> &'static str;
    fn datetime_type_sql(&self) -> &'static str;
    fn json_type_sql(&self) -> &'static str;

    // === SQL Generation ===
    fn limit_offset_sql(&self, limit: Option<u64>, offset: u64) -> String;
    fn current_timestamp_sql(&self) -> &'static str;

    // === Schema Introspection SQL ===
    fn list_tables_sql(&self, schema: Option<&str>) -> String;
    fn list_table_columns_sql(&self, table: &str, schema: Option<&str>) -> String;
    fn list_table_indexes_sql(&self, table: &str, schema: Option<&str>) -> String;

    // === DDL Generation ===
    fn create_table_sql(&self, table: &Table) -> Vec<String>;
    fn drop_table_sql(&self, table: &str) -> String;

    // === Transactions ===
    fn supports_savepoints(&self) -> bool { true }
    fn create_savepoint_sql(&self, name: &str) -> String;
    fn release_savepoint_sql(&self, name: &str) -> String;
    fn rollback_savepoint_sql(&self, name: &str) -> String;
}
```

**Consequences:**
- Klare Trennung zwischen Driver (Connection) und Platform (SQL)
- Associated Types für type-safe Statement/Result
- `Arc<dyn Platform>` für Sharing zwischen Connections

---

### ADR-005: Connection State Management

**Status:** Accepted

**Context:**
Doctrine Connection hat komplexen State (lazy connect, transaction nesting, rollback-only).

**Decision:**

```rust
pub struct Connection<D: Driver> {
    driver: D,
    inner: Option<D::Connection>,        // Lazy connection
    params: ConnectionParams,
    config: Configuration,

    // Transaction State
    transaction_nesting: AtomicU32,
    is_rollback_only: AtomicBool,
    auto_commit: bool,

    // Cached
    platform: OnceCell<Arc<dyn Platform>>,
}

impl<D: Driver> Connection<D> {
    /// Lazy connect - called internally on first query
    async fn ensure_connected(&mut self) -> Result<&mut D::Connection, Error> {
        if self.inner.is_none() {
            let conn = self.driver.connect(&self.params).await?;
            self.inner = Some(conn);

            if !self.auto_commit {
                self.begin_transaction_internal().await?;
            }
        }
        Ok(self.inner.as_mut().unwrap())
    }
}
```

**Transaction-Nesting mit Savepoints:**

```rust
impl<D: Driver> Connection<D> {
    pub async fn begin_transaction(&mut self) -> Result<(), Error> {
        let nesting = self.transaction_nesting.fetch_add(1, Ordering::SeqCst) + 1;
        let conn = self.ensure_connected().await?;

        if nesting == 1 {
            conn.execute("BEGIN").await?;
        } else {
            let savepoint = format!("RUSTINE_{}", nesting);
            conn.execute(&self.platform().create_savepoint_sql(&savepoint)).await?;
        }
        Ok(())
    }

    pub async fn commit(&mut self) -> Result<(), Error> {
        if self.is_rollback_only.load(Ordering::SeqCst) {
            return Err(TransactionError::RollbackOnly.into());
        }

        let nesting = self.transaction_nesting.fetch_sub(1, Ordering::SeqCst);
        if nesting == 0 {
            return Err(TransactionError::NoActiveTransaction.into());
        }

        let conn = self.inner.as_mut().ok_or(TransactionError::NoActiveTransaction)?;

        if nesting == 1 {
            conn.execute("COMMIT").await?;
            self.is_rollback_only.store(false, Ordering::SeqCst);
        } else {
            let savepoint = format!("RUSTINE_{}", nesting);
            conn.execute(&self.platform().release_savepoint_sql(&savepoint)).await?;
        }
        Ok(())
    }
}
```

**Consequences:**
- State ist explizit und nachvollziehbar
- Atomic Operations für Thread-Safety
- Savepoints transparent für nested transactions

---

### ADR-006: Type System Design

**Status:** Accepted

**Context:**
Doctrine hat ein Type-System für bidirektionale Konvertierung. Rust braucht Traits.

**Decision:**

```rust
// rustine-core/src/types.rs

/// Runtime-Value für SQL-Parameter und Ergebnisse
#[derive(Debug, Clone)]
pub enum SqlValue {
    Null,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    // Spezielle Typen
    Decimal(rust_decimal::Decimal),
    Uuid(uuid::Uuid),
    DateTime(chrono::NaiveDateTime),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    Json(serde_json::Value),
}

/// Trait für Types die zu SQL konvertiert werden können
pub trait ToSql: Send + Sync {
    fn to_sql(&self) -> SqlValue;
    fn sql_type_name() -> &'static str where Self: Sized;
}

/// Trait für Types die aus SQL gelesen werden können
pub trait FromSql: Sized {
    fn from_sql(value: SqlValue) -> Result<Self, ConversionError>;
}

// Blanket Implementations
impl ToSql for i32 {
    fn to_sql(&self) -> SqlValue { SqlValue::I32(*self) }
    fn sql_type_name() -> &'static str { "INTEGER" }
}

impl ToSql for String {
    fn to_sql(&self) -> SqlValue { SqlValue::String(self.clone()) }
    fn sql_type_name() -> &'static str { "TEXT" }
}

impl<T: ToSql> ToSql for Option<T> {
    fn to_sql(&self) -> SqlValue {
        match self {
            Some(v) => v.to_sql(),
            None => SqlValue::Null,
        }
    }
    fn sql_type_name() -> &'static str { T::sql_type_name() }
}
```

**Type Registry für Custom Types:**

```rust
pub struct TypeRegistry {
    types: HashMap<TypeId, Box<dyn SqlType>>,
}

pub trait SqlType: Send + Sync {
    fn name(&self) -> &'static str;
    fn to_sql(&self, value: &dyn Any) -> Result<SqlValue, ConversionError>;
    fn from_sql(&self, value: SqlValue) -> Result<Box<dyn Any>, ConversionError>;
    fn sql_declaration(&self, platform: &dyn Platform) -> String;
}
```

**Consequences:**
- Compile-time Type-Safety für Standard-Types
- Runtime-Flexibility für Custom Types
- Keine Allocations für primitive Types

---

### ADR-007: QueryBuilder Ownership Model

**Status:** Accepted

**Context:**
Doctrine's QueryBuilder ist mutable mit method chaining. Rust braucht klares Ownership.

**Decision:**
**Owned Builder Pattern mit `self` Consumption:**

```rust
pub struct QueryBuilder<'conn> {
    connection: &'conn Connection,
    state: QueryState,
}

enum QueryState {
    Select(SelectState),
    Insert(InsertState),
    Update(UpdateState),
    Delete(DeleteState),
}

struct SelectState {
    columns: Vec<String>,
    from: Vec<FromClause>,
    joins: Vec<JoinClause>,
    where_clause: Option<Expression>,
    group_by: Vec<String>,
    having: Option<Expression>,
    order_by: Vec<(String, Order)>,
    limit: Option<u64>,
    offset: u64,
    params: Parameters,
}

impl<'conn> QueryBuilder<'conn> {
    // Builder-Methods konsumieren self und geben Self zurück
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.state = QueryState::Select(SelectState::new(columns));
        self
    }

    pub fn from(mut self, table: &str, alias: Option<&str>) -> Self {
        if let QueryState::Select(ref mut state) = self.state {
            state.from.push(FromClause::new(table, alias));
        }
        self
    }

    pub fn and_where(mut self, expr: Expression) -> Self {
        if let QueryState::Select(ref mut state) = self.state {
            state.where_clause = Some(match state.where_clause.take() {
                Some(existing) => Expression::And(Box::new(existing), Box::new(expr)),
                None => expr,
            });
        }
        self
    }

    pub fn set_parameter<V: ToSql>(mut self, key: &str, value: V) -> Self {
        match &mut self.state {
            QueryState::Select(s) => s.params.set(key, value),
            // ... other states
        }
        self
    }

    // Terminal Operations
    pub async fn fetch_all(self) -> Result<Vec<Row>, Error> {
        let sql = self.to_sql()?;
        self.connection.fetch_all(&sql, &self.params()).await
    }

    pub async fn execute(self) -> Result<u64, Error> {
        let sql = self.to_sql()?;
        self.connection.execute_statement(&sql, &self.params()).await
    }
}
```

**Consequences:**
- Klares Ownership - Builder wird bei Terminal-Operation konsumiert
- Keine Clones nötig für Method-Chaining
- Compile-Error bei Reuse nach Consumption

---

## Implementation Patterns & Consistency Rules

### Naming Patterns

**Crate Naming:**
```
rustine-{component}     # z.B. rustine-core, rustine-driver
```

**Module Naming:**
```rust
// snake_case für Module
mod connection;
mod query_builder;
mod schema_manager;
```

**Type Naming:**
```rust
// PascalCase für Types
struct Connection<D: Driver> { ... }
struct QueryBuilder<'conn> { ... }
trait Platform { ... }
enum SqlValue { ... }
```

**Function Naming:**
```rust
// snake_case für Functions
fn execute_query() { ... }
fn quote_identifier() { ... }
fn list_tables() { ... }
```

**Constants:**
```rust
// SCREAMING_SNAKE_CASE
const DEFAULT_PORT: u16 = 5432;
const MAX_IDENTIFIER_LENGTH: usize = 63;
```

### Error Patterns

**Error Creation:**
```rust
// GOOD: Verwende Error-Variants
return Err(Error::Connection(ConnectionError::Refused(msg)));

// BAD: String-basierte Errors
return Err(Error::Query { message: "something failed".into(), sql: None });
```

**Error Context:**
```rust
// GOOD: Kontext hinzufügen
conn.execute(sql).await.map_err(|e| Error::Query {
    message: e.to_string(),
    sql: Some(sql.to_string()),
})?;

// BAD: Kontext verlieren
conn.execute(sql).await?;
```

### Async Patterns

**Async Function Signatures:**
```rust
// GOOD: Async wo I/O passiert
pub async fn execute(&self, sql: &str) -> Result<u64, Error>;

// BAD: Async ohne I/O
pub async fn quote_identifier(&self, id: &str) -> String; // Sollte sync sein
```

**Lifetime in Async:**
```rust
// GOOD: Explicit lifetime wenn nötig
pub async fn prepare<'a>(&'a self, sql: &str) -> Result<Statement<'a>, Error>;

// GOOD: Owned return um Lifetime-Probleme zu vermeiden
pub async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>, Error>;
```

### Documentation Patterns

**Public API Documentation:**
```rust
/// Executes a SQL query and returns all rows.
///
/// # Arguments
///
/// * `sql` - The SQL query to execute
/// * `params` - Parameters to bind to the query
///
/// # Returns
///
/// A vector of rows, or an error if the query fails.
///
/// # Example
///
/// ```rust
/// let rows = conn.fetch_all("SELECT * FROM users WHERE id = $1", &[&42]).await?;
/// ```
///
/// # Errors
///
/// Returns [`Error::Query`] if the SQL is invalid or execution fails.
pub async fn fetch_all(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Vec<Row>, Error>
```

### Testing Patterns

**Test Organization:**
```rust
// Unit tests im gleichen File
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_identifier() {
        let platform = PostgresPlatform::new();
        assert_eq!(platform.quote_identifier("user"), "\"user\"");
    }
}

// Integration tests in tests/ Verzeichnis
// tests/integration/postgres_connection.rs
#[tokio::test]
async fn test_connection_and_query() {
    let conn = test_connection().await;
    let result = conn.fetch_one("SELECT 1 as num").await.unwrap();
    assert_eq!(result.get::<i32>("num"), 1);
}
```

### Feature Flag Patterns

```toml
# Cargo.toml
[features]
default = ["postgres", "mysql", "sqlite"]

# Database backends
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
sqlite = ["sqlx/sqlite"]

# Optional features
tracing = ["dep:tracing"]
```

```rust
// Conditional compilation
#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "tracing")]
fn log_query(sql: &str) {
    tracing::debug!(sql = %sql, "Executing query");
}

#[cfg(not(feature = "tracing"))]
fn log_query(_sql: &str) {}
```

---

## Project Structure & Boundaries

### Complete Project Directory Structure

```
rustine/
├── Cargo.toml                      # Workspace definition
├── CLAUDE.md                       # AI assistant guidance
├── README.md                       # Project documentation
├── LICENSE-MIT                     # MIT License
├── LICENSE-APACHE                  # Apache 2.0 License
├── .github/
│   └── workflows/
│       ├── ci.yml                  # CI pipeline
│       └── release.yml             # Release automation
│
├── docs/
│   ├── prd.md                      # Product Requirements
│   ├── architecture.md             # This document
│   ├── 00-overview.md              # DBAL Analysis
│   ├── 01-connection.md
│   ├── 02-driver.md
│   ├── 03-platform.md
│   ├── 04-query-builder.md
│   ├── 05-schema.md
│   ├── 06-types.md
│   └── 07-rust-mapping.md
│
├── rustine/                        # Main crate (re-exports)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                  # pub use re-exports
│
├── rustine-core/                   # Core types, traits, errors
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs                # Error types
│       ├── types.rs                # SqlValue, ToSql, FromSql
│       ├── params.rs               # Parameters, ParameterType
│       ├── row.rs                  # Row type
│       └── config.rs               # Configuration types
│
├── rustine-driver/                 # Driver abstraction
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Driver traits
│       ├── connection.rs           # DriverConnection trait
│       ├── statement.rs            # DriverStatement trait
│       ├── result.rs               # DriverResult trait
│       ├── postgres/               # PostgreSQL driver
│       │   ├── mod.rs
│       │   ├── driver.rs
│       │   ├── connection.rs
│       │   └── statement.rs
│       ├── mysql/                  # MySQL driver
│       │   ├── mod.rs
│       │   ├── driver.rs
│       │   ├── connection.rs
│       │   └── statement.rs
│       └── sqlite/                 # SQLite driver
│           ├── mod.rs
│           ├── driver.rs
│           ├── connection.rs
│           └── statement.rs
│
├── rustine-platform/               # SQL dialect abstraction
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Platform trait
│       ├── postgres.rs             # PostgreSQL platform
│       ├── mysql.rs                # MySQL platform
│       ├── sqlite.rs               # SQLite platform
│       └── keywords/               # Reserved keywords per platform
│           ├── mod.rs
│           ├── postgres.rs
│           └── mysql.rs
│
├── rustine-query/                  # QueryBuilder
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── builder.rs              # QueryBuilder struct
│       ├── select.rs               # SELECT state/logic
│       ├── insert.rs               # INSERT state/logic
│       ├── update.rs               # UPDATE state/logic
│       ├── delete.rs               # DELETE state/logic
│       ├── expression.rs           # Expression builder
│       └── parts/                  # Query parts
│           ├── mod.rs
│           ├── from.rs             # FROM clause
│           ├── join.rs             # JOIN clauses
│           ├── where_clause.rs     # WHERE clause
│           └── order.rs            # ORDER BY
│
├── rustine-schema/                 # Schema introspection
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── manager.rs              # SchemaManager trait
│       ├── schema.rs               # Schema struct
│       ├── table.rs                # Table struct
│       ├── column.rs               # Column struct
│       ├── index.rs                # Index struct
│       ├── foreign_key.rs          # ForeignKey struct
│       ├── comparator.rs           # Schema diff
│       └── introspection/          # Platform-specific introspection
│           ├── mod.rs
│           ├── postgres.rs
│           ├── mysql.rs
│           └── sqlite.rs
│
├── rustine-derive/                 # Proc macros (später)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── tests/                          # Integration tests
│   ├── common/
│   │   └── mod.rs                  # Test utilities
│   ├── postgres/
│   │   ├── connection.rs
│   │   ├── query_builder.rs
│   │   └── schema.rs
│   ├── mysql/
│   │   └── ...
│   └── sqlite/
│       └── ...
│
├── benches/                        # Benchmarks
│   ├── query_execution.rs
│   └── query_builder.rs
│
└── examples/
    ├── basic_query.rs
    ├── query_builder.rs
    ├── transactions.rs
    └── schema_introspection.rs
```

### Architectural Boundaries

**Crate Dependencies:**
```
rustine (meta-crate)
    ├── rustine-core
    ├── rustine-driver
    │   └── rustine-core
    ├── rustine-platform
    │   └── rustine-core
    ├── rustine-query
    │   ├── rustine-core
    │   └── rustine-platform
    └── rustine-schema
        ├── rustine-core
        └── rustine-platform
```

**Layering Rules:**
- `rustine-core` hat KEINE Abhängigkeiten zu anderen rustine-* Crates
- `rustine-driver` und `rustine-platform` kennen nur `rustine-core`
- `rustine-query` kennt `rustine-core` und `rustine-platform`
- `rustine-schema` kennt `rustine-core` und `rustine-platform`
- Nur `rustine` (meta) kennt alle Crates

### FR → Crate Mapping

| FR-Bereich | Crate |
|------------|-------|
| FR1-FR5 (Connection) | `rustine-driver`, `rustine` |
| FR6-FR12 (Query Execution) | `rustine-core`, `rustine-driver` |
| FR13-FR17 (Transactions) | `rustine-driver` |
| FR18-FR27 (QueryBuilder) | `rustine-query` |
| FR28-FR32 (Platform) | `rustine-platform` |
| FR33-FR38 (Schema) | `rustine-schema` |
| FR39-FR45 (Types) | `rustine-core` |
| FR46-FR49 (Errors) | `rustine-core` |

---

## Validation Checklist

### Architecture Coherence

- [x] Alle 49 FRs sind einem Crate zugeordnet
- [x] Keine zirkulären Abhängigkeiten zwischen Crates
- [x] Async-Strategie ist konsistent durchgezogen
- [x] Error-Handling folgt einheitlichem Pattern
- [x] Trait-Design ermöglicht Erweiterbarkeit

### Technology Alignment

- [x] Tokio als Runtime passt zu sqlx
- [x] MSRV 1.75 unterstützt async fn in traits
- [x] Feature-Flags sind sinnvoll definiert
- [x] Keine unsafe Code in Public API geplant

### Implementation Readiness

- [x] Core Traits sind definiert
- [x] Error Types sind vollständig
- [x] Project Structure ist konkret
- [x] Naming Conventions sind festgelegt
- [x] Testing Strategy ist definiert

---

## Next Steps

Nach dem Architecture Document:

1. **Epic Breakdown** - FRs in implementierbare Stories aufteilen
2. **Implementation Phase 1** - `rustine-core` implementieren
3. **Implementation Phase 2** - `rustine-driver` (SQLite zuerst)
4. **Implementation Phase 3** - `rustine-platform` + `rustine-query`
5. **Implementation Phase 4** - `rustine-schema`

---

*Dieses Architecture Document basiert auf der BMAD Method v6 und definiert die technische Basis für die Rustine-Implementierung.*
