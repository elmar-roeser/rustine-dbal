# Rust-Portierungs-Strategie

Dieses Dokument fasst die Strategien für die Portierung von Doctrine DBAL nach Rust zusammen.

## PHP → Rust Pattern-Mapping

| PHP Pattern | Rust Äquivalent |
|-------------|-----------------|
| Abstract Class | Trait + Default Impl |
| Interface | Trait |
| Factory (static) | Builder Pattern / `::new()` |
| Exception Hierarchy | Error Enum + thiserror |
| Nullable `?Type` | `Option<T>` |
| Mixed Return | `Result<T, E>` oder Enum |
| Lazy Loading | `OnceCell<T>` / `Lazy<T>` |
| Inheritance | Trait Composition / Generics |
| Protected Properties | `pub(crate)` |
| Type Hints Array | `Vec<T>`, `HashMap<K,V>` |

## Crate-Struktur

```
rustine/
├── Cargo.toml                    # Workspace
├── rustine/                      # Main crate (re-exports)
│   └── Cargo.toml
├── rustine-core/                 # Core traits, types, errors
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs              # Error types
│       ├── types.rs              # SqlType trait, SqlValue
│       ├── params.rs             # ParameterType, Parameters
│       └── config.rs             # Configuration
├── rustine-driver/               # Driver abstraction
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── driver.rs             # Driver trait
│       ├── connection.rs         # DriverConnection trait
│       ├── statement.rs          # DriverStatement trait
│       ├── result.rs             # DriverResult trait
│       ├── postgres/             # PostgreSQL impl (sqlx/tokio-postgres)
│       ├── mysql/                # MySQL impl
│       └── sqlite/               # SQLite impl
├── rustine-platform/             # SQL dialect abstraction
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── platform.rs           # Platform trait
│       ├── postgres.rs
│       ├── mysql.rs
│       └── sqlite.rs
├── rustine-schema/               # Schema introspection & management
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── schema.rs             # Schema, Table, Column, etc.
│       ├── manager.rs            # SchemaManager trait
│       ├── comparator.rs         # Schema diff
│       └── introspection/
├── rustine-query/                # QueryBuilder
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── builder.rs            # QueryBuilder
│       ├── expression.rs         # ExpressionBuilder
│       └── parts.rs              # From, Join, etc.
└── rustine-derive/               # Proc macros
    ├── Cargo.toml
    └── src/
        └── lib.rs                # #[derive(SqlType)], etc.
```

## Core Traits

### Error Handling

```rust
// rustine-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Query error: {message}")]
    Query { message: String, sql: Option<String> },

    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),

    #[error("Type conversion error: {0}")]
    Conversion(String),

    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),

    #[error("Driver error: {source}")]
    Driver { source: Box<dyn std::error::Error + Send + Sync> },
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection lost")]
    Lost,

    #[error("Connection refused: {0}")]
    Refused(String),

    #[error("Authentication failed")]
    AuthFailed,
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("No active transaction")]
    NoActiveTransaction,

    #[error("Transaction marked rollback-only")]
    RollbackOnly,

    #[error("Savepoints not supported")]
    SavepointsNotSupported,
}
```

### Connection

```rust
// rustine-core/src/connection.rs
use async_trait::async_trait;

pub struct Connection<D: Driver> {
    driver: D,
    inner: Option<D::Connection>,
    config: Configuration,
    params: ConnectionParams,

    auto_commit: bool,
    transaction_nesting: u32,
    is_rollback_only: bool,

    platform: OnceCell<Arc<dyn Platform>>,
}

impl<D: Driver> Connection<D> {
    pub async fn new(params: ConnectionParams, driver: D, config: Configuration) -> Result<Self, Error> {
        Ok(Self {
            driver,
            inner: None,
            config,
            params,
            auto_commit: true,
            transaction_nesting: 0,
            is_rollback_only: false,
            platform: OnceCell::new(),
        })
    }

    async fn connect(&mut self) -> Result<&D::Connection, Error> {
        if self.inner.is_none() {
            let conn = self.driver.connect(&self.params).await?;
            self.inner = Some(conn);

            if !self.auto_commit {
                self.begin_transaction().await?;
            }
        }
        Ok(self.inner.as_ref().unwrap())
    }

    pub fn is_connected(&self) -> bool {
        self.inner.is_some()
    }

    pub async fn close(&mut self) {
        self.inner = None;
        self.transaction_nesting = 0;
    }

    // Query execution
    pub async fn execute_query(
        &mut self,
        sql: &str,
        params: impl Into<Parameters>,
    ) -> Result<impl Stream<Item = Result<Row, Error>>, Error> {
        let conn = self.connect().await?;
        // ...
    }

    pub async fn execute_statement(
        &mut self,
        sql: &str,
        params: impl Into<Parameters>,
    ) -> Result<u64, Error> {
        let conn = self.connect().await?;
        // ...
    }

    // Transactions
    pub async fn begin_transaction(&mut self) -> Result<(), Error> {
        self.connect().await?;
        self.transaction_nesting += 1;

        if self.transaction_nesting == 1 {
            self.inner.as_mut().unwrap().begin_transaction().await?;
        } else {
            let savepoint = format!("RUSTINE_{}", self.transaction_nesting);
            self.create_savepoint(&savepoint).await?;
        }
        Ok(())
    }

    pub async fn commit(&mut self) -> Result<(), Error> {
        if self.transaction_nesting == 0 {
            return Err(TransactionError::NoActiveTransaction.into());
        }
        if self.is_rollback_only {
            return Err(TransactionError::RollbackOnly.into());
        }

        if self.transaction_nesting == 1 {
            self.inner.as_mut().unwrap().commit().await?;
        } else {
            let savepoint = format!("RUSTINE_{}", self.transaction_nesting);
            self.release_savepoint(&savepoint).await?;
        }
        self.transaction_nesting -= 1;
        Ok(())
    }

    pub async fn rollback(&mut self) -> Result<(), Error> {
        if self.transaction_nesting == 0 {
            return Err(TransactionError::NoActiveTransaction.into());
        }

        if self.transaction_nesting == 1 {
            self.inner.as_mut().unwrap().rollback().await?;
            self.is_rollback_only = false;
        } else {
            let savepoint = format!("RUSTINE_{}", self.transaction_nesting);
            self.rollback_savepoint(&savepoint).await?;
        }
        self.transaction_nesting -= 1;
        Ok(())
    }

    pub async fn transactional<T, F, Fut>(&mut self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Self) -> Fut,
        Fut: Future<Output = Result<T, Error>>,
    {
        self.begin_transaction().await?;

        match f(self).await {
            Ok(result) => {
                self.commit().await?;
                Ok(result)
            }
            Err(e) => {
                self.rollback().await?;
                Err(e)
            }
        }
    }

    // Factories
    pub fn query_builder(&self) -> QueryBuilder<'_> {
        QueryBuilder::new(self)
    }

    pub fn schema_manager(&self) -> impl SchemaManager + '_ {
        // Platform-spezifischen SchemaManager zurückgeben
    }

    pub fn platform(&self) -> &dyn Platform {
        self.platform.get_or_init(|| {
            self.driver.platform(&self.server_version())
        }).as_ref()
    }
}
```

## Feature Flags

```toml
# rustine/Cargo.toml
[features]
default = ["postgres", "mysql", "sqlite"]

# Datenbank-Backends
postgres = ["rustine-driver/postgres"]
mysql = ["rustine-driver/mysql"]
sqlite = ["rustine-driver/sqlite"]
mssql = ["rustine-driver/mssql"]

# Runtime
runtime-tokio = ["tokio"]
runtime-async-std = ["async-std"]

# Extras
tracing = ["dep:tracing"]
```

## API-Design Beispiele

### Connection aufbauen

```rust
use rustine::prelude::*;

// Mit Builder
let conn = Connection::builder()
    .driver(PostgresDriver::new())
    .host("localhost")
    .port(5432)
    .database("myapp")
    .username("user")
    .password("secret")
    .build()
    .await?;

// Oder mit Connection String
let conn = Connection::from_url("postgres://user:secret@localhost/myapp").await?;
```

### Queries ausführen

```rust
// Einfache Query
let rows = conn.fetch_all("SELECT * FROM users WHERE active = $1", &[&true]).await?;

// Mit QueryBuilder
let users = conn.query_builder()
    .select(&["id", "name", "email"])
    .from("users", Some("u"))
    .where_clause(expr::eq("u.active", "$1"))
    .order_by("u.name", Order::Asc)
    .limit(10)
    .set_parameter(1, true)
    .fetch_all()
    .await?;

// INSERT
let affected = conn.insert("users", &[
    ("name", &"John" as &dyn ToSql),
    ("email", &"john@example.com"),
]).await?;

// Mit QueryBuilder
conn.query_builder()
    .insert("users")
    .values(&[
        ("name", ":name"),
        ("email", ":email"),
    ])
    .set_parameter("name", "John")
    .set_parameter("email", "john@example.com")
    .execute()
    .await?;
```

### Transaktionen

```rust
// Explizit
conn.begin_transaction().await?;
conn.execute("INSERT INTO users (name) VALUES ($1)", &[&"John"]).await?;
conn.execute("INSERT INTO profiles (user_id) VALUES ($1)", &[&user_id]).await?;
conn.commit().await?;

// Mit Closure
let user_id = conn.transactional(|conn| async move {
    let id = conn.insert("users", &[("name", &"John")]).await?;
    conn.insert("profiles", &[("user_id", &id)]).await?;
    Ok(id)
}).await?;
```

### Schema-Management

```rust
let schema_manager = conn.schema_manager();

// Introspection
let tables = schema_manager.list_table_names().await?;
let users_table = schema_manager.introspect_table("users").await?;

// Schema erstellen
let table = Table::builder("posts")
    .add_column(Column::integer("id").auto_increment())
    .add_column(Column::integer("user_id").not_null())
    .add_column(Column::string("title", 255).not_null())
    .add_column(Column::text("content"))
    .add_column(Column::datetime("created_at").default_now())
    .primary_key(&["id"])
    .foreign_key(
        ForeignKey::new(&["user_id"], "users", &["id"])
            .on_delete(ReferentialAction::Cascade)
    )
    .build();

schema_manager.create_table(&table).await?;

// Migration
let from_schema = schema_manager.introspect_schema().await?;
let to_schema = /* desired schema */;
let diff = Comparator::compare_schemas(&from_schema, &to_schema);
let sql_statements = conn.platform().alter_schema_sql(&diff);
```

## Implementierungs-Reihenfolge

### Phase 1: Core Foundation
1. `rustine-core`: Error types, Configuration, ParameterType
2. `rustine-driver`: Driver traits (ohne Implementierungen)
3. Erste Driver-Implementierung: SQLite (einfachster Fall)

### Phase 2: SQL Abstraction
4. `rustine-platform`: Platform trait, SQLite Platform
5. `rustine-query`: QueryBuilder basics (SELECT)
6. PostgreSQL Driver + Platform

### Phase 3: Schema Management
7. `rustine-schema`: Schema objects (Table, Column, Index)
8. SchemaManager trait + Introspection
9. Schema Comparator

### Phase 4: Full Feature Set
10. Alle QueryBuilder-Features (INSERT, UPDATE, DELETE, UNION, CTE)
11. MySQL Driver + Platform
12. Type System vollständig
13. Derive Macros

### Phase 5: Polish
14. Documentation
15. Benchmarks
16. Integration Tests mit echten Datenbanken
