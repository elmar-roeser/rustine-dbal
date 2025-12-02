# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Projektziel

**Rustine** ist eine Rust-Portierung von Doctrine DBAL (und später ORM). Ziel ist es, die Fähigkeiten und Architekturkonzepte von Doctrine in idiomatic Rust zu übertragen - keine 1:1 Kopie, sondern eine Neuimplementierung mit modernen Rust-Patterns.

Das `dbal/` Verzeichnis enthält den Doctrine DBAL PHP-Quellcode als Referenz für die Analyse.

## Referenz: Doctrine DBAL (PHP)

### Quellcode-Analyse Commands

```bash
# PHP-Code im dbal/ Verzeichnis analysieren
cd dbal

# Tests ausführen (für Verhaltensverständnis)
vendor/bin/phpunit

# Spezifische Plattform-Tests
vendor/bin/phpunit -c ci/github/phpunit/pdo_sqlite.xml
```

### Doctrine DBAL Architektur

#### Core Components (zu portieren)

| PHP Component | Pfad | Rust-Äquivalent Idee |
|---------------|------|----------------------|
| Connection | `dbal/src/Connection.php` | `struct Connection<D: Driver>` |
| DriverManager | `dbal/src/DriverManager.php` | Builder-Pattern oder `ConnectionConfig` |
| Driver trait | `dbal/src/Driver.php` | `trait Driver` |
| Platform | `dbal/src/Platforms/AbstractPlatform.php` | `trait Platform` + `enum DatabasePlatform` |
| SchemaManager | `dbal/src/Schema/AbstractSchemaManager.php` | `trait SchemaManager` |
| QueryBuilder | `dbal/src/Query/QueryBuilder.php` | Builder mit owned types |
| Type System | `dbal/src/Types/` | `trait SqlType` + derive macros |

#### Layer-Struktur

```
┌─────────────────────────────────────────────────┐
│  QueryBuilder / Schema API                       │
├─────────────────────────────────────────────────┤
│  Connection (Transaction, Query Execution)       │
├─────────────────────────────────────────────────┤
│  Platform (SQL-Dialekt Generation)               │
├─────────────────────────────────────────────────┤
│  Driver (Connection, Statement, Result)          │
├─────────────────────────────────────────────────┤
│  Native DB Client (sqlx, tokio-postgres, etc.)   │
└─────────────────────────────────────────────────┘
```

### Doctrine Features zur Analyse

**Connection & Transactions:**
- Lazy connection (connect on first query)
- Transaction nesting mit Savepoints
- Auto-commit Modus
- Connection pooling (via Middleware)

**Query Execution:**
- Prepared Statements mit Parameter-Binding
- Named und positional Parameters
- Array-Parameter Expansion (`IN (?)` mit Arrays)
- Result iteration (fetch, fetchAll, fetchColumn, etc.)

**Query Builder:**
- SELECT, INSERT, UPDATE, DELETE
- JOINs, Subqueries, UNION
- Expression Builder für Conditions
- Parameter-Binding integration

**Schema Management:**
- Introspection (Tables, Columns, Indexes, Foreign Keys, Sequences)
- Schema Diff & Migration Generation
- Schema Creation/Modification

**Type System:**
- Bidirektionale Konvertierung (PHP ↔ DB)
- Custom Types registrierbar
- Platform-spezifische SQL-Type-Mappings

**Platform Abstraction:**
- SQL-Dialekt Unterschiede (LIMIT, Quoting, Date-Funktionen)
- DDL-Generation (CREATE TABLE, ALTER, etc.)
- Platform-Detection

## Rust-Portierung Richtlinien

### Empfohlene Rust-Patterns

```rust
// Statt PHP Vererbung → Traits + Generics
trait Platform {
    fn get_list_tables_sql(&self) -> String;
    fn quote_identifier(&self, name: &str) -> String;
}

// Statt Factory-Pattern → Builder
let conn = Connection::builder()
    .driver(PostgresDriver::new())
    .host("localhost")
    .database("mydb")
    .build()?;

// Statt Exception-Hierarchie → Error enum + thiserror
#[derive(Error, Debug)]
enum DbalError {
    #[error("Connection failed: {0}")]
    Connection(#[from] ConnectionError),
    #[error("Query failed: {0}")]
    Query(String),
}

// Async-first Design
async fn execute(&self, sql: &str) -> Result<u64, DbalError>;
```

### Crate-Struktur Vorschlag

```
rustine/
├── rustine-core/        # Traits, Types, Error definitions
├── rustine-driver/      # Driver trait + implementations
│   ├── postgres/
│   ├── mysql/
│   └── sqlite/
├── rustine-platform/    # SQL dialect implementations
├── rustine-schema/      # Schema introspection & management
├── rustine-query/       # QueryBuilder
└── rustine/             # Re-exports, convenience API
```

### Rust-Ökosystem Integration

| Funktion | Empfohlene Crates |
|----------|-------------------|
| Async Runtime | `tokio` |
| DB Clients | `sqlx`, `tokio-postgres`, `mysql_async`, `rusqlite` |
| Error Handling | `thiserror`, `anyhow` |
| Serialization | `serde` |
| Derive Macros | `proc-macro2`, `syn`, `quote` |
| Connection Pool | `deadpool`, `bb8`, oder sqlx built-in |

## Projekt-Dokumentation

### Product Requirements Document (PRD)

**[docs/prd.md](docs/prd.md)** - Basierend auf BMAD Method v6

Enthält:
- Executive Summary & Vision
- Success Criteria & Scope Definition
- Developer Journeys
- 49 Functional Requirements
- Non-Functional Requirements

### Architecture Decision Document

**[docs/architecture.md](docs/architecture.md)** - Basierend auf BMAD Method v6

Enthält:
- 7 Architecture Decision Records (ADRs)
- Crate-Struktur & Dependencies
- Trait-Designs (Driver, Platform, Types)
- Implementation Patterns & Naming Conventions
- Vollständige Project Structure
- FR → Crate Mapping

### Epic Breakdown

**[docs/epics.md](docs/epics.md)** - Basierend auf BMAD Method v6

Enthält:
- 6 Epics mit 29 Stories
- Alle 49 FRs abgedeckt
- Acceptance Criteria (Given/When/Then)
- Implementierungs-Reihenfolge & Dependency Graph

### Doctrine DBAL Analyse

| Dokument | Inhalt |
|----------|--------|
| [00-overview.md](docs/00-overview.md) | Architektur-Übersicht, Schichten, Datenfluss |
| [01-connection.md](docs/01-connection.md) | Connection-Klasse: State, API, Transactions |
| [02-driver.md](docs/02-driver.md) | Driver-Layer: Interfaces, Implementierungen |
| [03-platform.md](docs/03-platform.md) | SQL-Dialekte: Type-Deklarationen, DDL, DML |
| [04-query-builder.md](docs/04-query-builder.md) | QueryBuilder: SELECT, INSERT, UPDATE, DELETE |
| [05-schema.md](docs/05-schema.md) | Schema-Management: Introspection, DDL, Diff |
| [06-types.md](docs/06-types.md) | Type-System: Konvertierung, Custom Types |
| [07-rust-mapping.md](docs/07-rust-mapping.md) | Rust-Portierungs-Strategie, API-Design |

## Unterstützte Datenbanken (Ziel)

Priorität 1: PostgreSQL, SQLite, MySQL/MariaDB
Priorität 2: SQL Server (via tiberius)
Später: Oracle, DB2 (falls Rust-Treiber verfügbar)

## Implementierungs-Status

### Phase 1: Core Foundation (IN PROGRESS)

**rustine-core** - IMPLEMENTIERT:
- Error types (Error, ConnectionError, TransactionError, QueryError, SchemaError)
- SqlValue enum mit Feature-Flags (chrono, uuid, json, decimal)
- ToSql / FromSql traits mit Implementierungen
- Configuration (ConnectionParams, Configuration, IsolationLevel)
- ParameterType enum
- 38 Unit Tests, alle bestanden

**rustine-driver** - STRUKTUR:
- Driver, DriverConnection, DriverStatement, DriverResult traits definiert

**rustine-platform** - STRUKTUR:
- Platform trait definiert
- PostgresPlatform, MySqlPlatform, SqlitePlatform Grundstruktur
- 5 Unit Tests

### Nächste Schritte

1. **Epic 2 - Database Connectivity**: SQLite Driver implementieren
2. **Epic 3 - Transactions**: Transaction Management in Connection
3. **Epic 4 - Platform Abstraction**: Vollständige Platform-Implementierungen
4. **Epic 5 - Query Builder**: SELECT, INSERT, UPDATE, DELETE
5. **Epic 6 - Schema Introspection**: SchemaManager, Table, Column

## Build Commands

```bash
# Alle Tests ausführen
cargo test --workspace

# Nur rustine-core testen
cargo test -p rustine-core

# Build
cargo build --workspace

# Clippy
cargo clippy --workspace

# Documentation generieren
cargo doc --workspace --no-deps --open
```

## Git Workflow

### Conventional Commits

Alle Commits folgen dem [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) Standard:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**

| Type | Beschreibung |
|------|--------------|
| `feat` | Neues Feature |
| `fix` | Bugfix |
| `docs` | Dokumentation |
| `style` | Formatierung (kein Code-Change) |
| `refactor` | Code-Refactoring |
| `perf` | Performance-Verbesserung |
| `test` | Tests hinzufügen/ändern |
| `build` | Build-System, Dependencies |
| `ci` | CI/CD Konfiguration |
| `chore` | Sonstige Änderungen |

**Scopes (optional):**
- `core` - rustine-core
- `driver` - rustine-driver
- `platform` - rustine-platform
- `query` - rustine-query
- `schema` - rustine-schema

**Beispiele:**

```bash
feat(core): add SqlValue conversion for chrono types
fix(driver): handle connection timeout correctly
docs: update PRD with new requirements
test(platform): add PostgreSQL quoting tests
refactor(query): simplify WHERE clause building
```

**Breaking Changes:**

```bash
feat(core)!: change SqlValue enum structure

BREAKING CHANGE: SqlValue variants renamed for clarity
```

### Versionierung

Folgt [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking Changes
- **MINOR** (0.1.0): Neue Features (rückwärtskompatibel)
- **PATCH** (0.0.1): Bugfixes

Aktuell: **v0.1.0**

### Changelog

Der [CHANGELOG.md](CHANGELOG.md) folgt dem [Keep a Changelog](https://keepachangelog.com/) Format.

**Nach jedem Arbeitsschritt:**

1. CHANGELOG.md unter `[Unreleased]` aktualisieren
2. Kategorien verwenden:
   - `Added` - Neue Features
   - `Changed` - Änderungen an bestehenden Features
   - `Deprecated` - Features die bald entfernt werden
   - `Removed` - Entfernte Features
   - `Fixed` - Bugfixes
   - `Security` - Sicherheitsfixes

**Bei Release:**

1. `[Unreleased]` → `[X.Y.Z] - YYYY-MM-DD` umbenennen
2. Neuen `[Unreleased]` Abschnitt hinzufügen
3. Version in allen `Cargo.toml` aktualisieren
4. Git Tag erstellen: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`

### Release-Workflow

```bash
# Version bump in Cargo.toml (workspace)
# CHANGELOG.md aktualisieren
git add -A
git commit -m "chore: release vX.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main --tags
```
