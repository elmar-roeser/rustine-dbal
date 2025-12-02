# rustine-dbal

[![Crates.io](https://img.shields.io/crates/v/rustine-dbal.svg)](https://crates.io/crates/rustine-dbal)
[![Documentation](https://docs.rs/rustine-dbal/badge.svg)](https://docs.rs/rustine-dbal)
[![License](https://img.shields.io/crates/l/rustine-dbal.svg)](LICENSE)

**Rustine DBAL** is an idiomatic Rust Database Abstraction Layer, inspired by [Doctrine DBAL](https://www.doctrine-project.org/projects/dbal.html).

## Features

- **Connection Management** - Lazy connections with transaction support
- **Transaction Support** - ACID transactions with savepoint-based nesting
- **Driver Abstraction** - Unified API for different databases
- **Type System** - Bidirectional conversion between Rust and SQL types
- **Platform Abstraction** - SQL dialect differences abstracted away

## Installation

```toml
[dependencies]
rustine-dbal = { version = "0.2", features = ["sqlite"] }
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| `sqlite` | SQLite driver via sqlx |
| `chrono` | Date/time support (default) |
| `uuid` | UUID support (default) |
| `json` | JSON support (default) |
| `decimal` | Decimal number support (default) |
| `tracing` | Logging via tracing |

## Quick Start

```rust
use rustine_dbal::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // SQLite in-memory database
    let driver = SqliteDriver::new();
    let params = ConnectionParams::sqlite_memory();
    let conn = Connection::new(&driver, &params).await?;

    // Create table
    conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").await?;

    // Insert data with transaction
    conn.begin_transaction().await?;
    conn.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
    conn.execute("INSERT INTO users (name) VALUES ('Bob')").await?;
    conn.commit().await?;

    // Query data
    let mut result = conn.query("SELECT * FROM users").await?;
    for row in result.all_rows()? {
        println!("{:?}", row);
    }

    Ok(())
}
```

## Transaction Management

### Simple Transactions

```rust
conn.begin_transaction().await?;
conn.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
conn.commit().await?;
// or: conn.rollback().await?;
```

### Nested Transactions (Savepoints)

```rust
conn.begin_transaction().await?;           // Level 1
conn.execute("INSERT ...").await?;

conn.begin_transaction().await?;           // Level 2 (SAVEPOINT RUSTINE_1)
conn.execute("INSERT ...").await?;
conn.rollback().await?;                    // ROLLBACK TO SAVEPOINT

conn.commit().await?;                      // Commit Level 1
```

### Transaction State

```rust
conn.is_transaction_active()     // true if TX active
conn.transaction_nesting_level() // 0, 1, 2, ...
conn.is_rollback_only()          // true if only rollback possible
conn.set_rollback_only()         // Mark TX as rollback-only
```

## Architecture

```
rustine-dbal/
├── core/           # Core types (Error, SqlValue, ToSql, FromSql)
├── connection/     # High-level Connection with TX management
├── driver/         # Driver traits and implementations
│   └── sqlite/     # SQLite driver
├── platform/       # SQL dialect abstractions
├── query/          # Query Builder (planned)
└── schema/         # Schema introspection (planned)
```

## Roadmap

- [x] **Epic 1**: Core Foundation (Types, Errors, Traits)
- [x] **Epic 2**: SQLite Driver
- [x] **Epic 3**: Transaction Management
- [ ] **Epic 4**: Platform Abstraction (PostgreSQL, MySQL dialects)
- [ ] **Epic 5**: Query Builder
- [ ] **Epic 6**: Schema Introspection

## Documentation

- [CHANGELOG](CHANGELOG.md) - Version history
- [API Docs](https://docs.rs/rustine-dbal) - Rust documentation

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

## Contributing

Contributions are welcome! Please note:

- [Conventional Commits](https://www.conventionalcommits.org/) for commit messages
- [Semantic Versioning](https://semver.org/) for versioning
