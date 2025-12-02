# rustine-dbal

[![Crates.io](https://img.shields.io/crates/v/rustine-dbal.svg)](https://crates.io/crates/rustine-dbal)
[![Documentation](https://docs.rs/rustine-dbal/badge.svg)](https://docs.rs/rustine-dbal)
[![License](https://img.shields.io/crates/l/rustine-dbal.svg)](LICENSE)

**Rustine DBAL** ist eine idiomatische Rust Database Abstraction Layer, inspiriert von [Doctrine DBAL](https://www.doctrine-project.org/projects/dbal.html).

## Features

- **Connection Management** - Lazy Connections, Transaction-Support
- **Transaction Support** - ACID Transactions mit Savepoint-basiertem Nesting
- **Driver Abstraktion** - Einheitliche API für verschiedene Datenbanken
- **Type System** - Bidirektionale Konvertierung zwischen Rust und SQL Typen
- **Platform Abstraction** - SQL-Dialekt-Unterschiede abstrahiert

## Installation

```toml
[dependencies]
rustine-dbal = { version = "0.2", features = ["sqlite"] }
```

### Feature Flags

| Feature | Beschreibung |
|---------|-------------|
| `sqlite` | SQLite-Treiber via sqlx |
| `chrono` | Datum/Zeit-Support (default) |
| `uuid` | UUID-Support (default) |
| `json` | JSON-Support (default) |
| `decimal` | Dezimalzahlen-Support (default) |
| `tracing` | Logging via tracing |

## Schnellstart

```rust
use rustine_dbal::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // SQLite In-Memory Datenbank
    let driver = SqliteDriver::new();
    let params = ConnectionParams::sqlite_memory();
    let conn = Connection::new(&driver, &params).await?;

    // Tabelle erstellen
    conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").await?;

    // Daten einfügen mit Transaction
    conn.begin_transaction().await?;
    conn.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
    conn.execute("INSERT INTO users (name) VALUES ('Bob')").await?;
    conn.commit().await?;

    // Daten abfragen
    let mut result = conn.query("SELECT * FROM users").await?;
    for row in result.all_rows()? {
        println!("{:?}", row);
    }

    Ok(())
}
```

## Transaction Management

### Einfache Transactions

```rust
conn.begin_transaction().await?;
conn.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
conn.commit().await?;
// oder: conn.rollback().await?;
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
conn.is_transaction_active()   // true wenn TX aktiv
conn.transaction_nesting_level() // 0, 1, 2, ...
conn.is_rollback_only()        // true wenn nur Rollback möglich
conn.set_rollback_only()       // Markiert TX als rollback-only
```

## Architektur

```
rustine-dbal/
├── core/           # Grundtypen (Error, SqlValue, ToSql, FromSql)
├── connection/     # High-Level Connection mit TX-Management
├── driver/         # Driver-Traits und Implementierungen
│   └── sqlite/     # SQLite-Treiber
├── platform/       # SQL-Dialekt-Abstraktionen
├── query/          # Query Builder (geplant)
└── schema/         # Schema-Introspection (geplant)
```

## Roadmap

- [x] **Epic 1**: Core Foundation (Types, Errors, Traits)
- [x] **Epic 2**: SQLite Driver
- [x] **Epic 3**: Transaction Management
- [ ] **Epic 4**: Platform Abstraction (PostgreSQL, MySQL Dialekte)
- [ ] **Epic 5**: Query Builder
- [ ] **Epic 6**: Schema Introspection

## Dokumentation

- [CHANGELOG](CHANGELOG.md) - Versionshistorie
- [API Docs](https://docs.rs/rustine-dbal) - Rust-Dokumentation

## Lizenz

Dual-lizenziert unter [MIT](LICENSE-MIT) oder [Apache-2.0](LICENSE-APACHE).

## Beitragen

Contributions sind willkommen! Bitte beachte:

- [Conventional Commits](https://www.conventionalcommits.org/) für Commit-Messages
- [Semantic Versioning](https://semver.org/) für Versionierung
