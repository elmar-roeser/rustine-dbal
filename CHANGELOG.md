# Changelog

Alle wichtigen Änderungen an diesem Projekt werden in dieser Datei dokumentiert.

Das Format basiert auf [Keep a Changelog](https://keepachangelog.com/de/1.1.0/),
und dieses Projekt folgt [Semantic Versioning](https://semver.org/lang/de/).

## [Unreleased]

### Added
- **SQLite Driver** (Epic 2: Database Connectivity)
  - `SqliteDriver` - Treiber-Implementierung mit sqlx
  - `SqliteConnection` - Verbindungsmanagement mit echter einzelner Connection (nicht Pool)
  - `SqliteStatement` - Prepared Statements mit positional/named Parameters
  - `SqliteResult` - Ergebnisset-Iteration
  - Feature-Flag `sqlite` für optionale Aktivierung
  - Verbesserte Typ-Erkennung für dynamische SQLite-Ausdrücke (COUNT(*), etc.)

- **Transaction Management** (Epic 3)
  - `Connection<D>` - High-Level Connection-Wrapper über Driver-Traits
  - Nested Transactions via Savepoints (`RUSTINE_1`, `RUSTINE_2`, etc.)
  - `begin_transaction()`, `commit()`, `rollback()` mit Nesting-Counter
  - `transactional_boxed()` - Transaktionale Closure-API mit auto-commit/rollback
  - `in_transaction()` - Einfaches Result-basiertes Commit/Rollback
  - Transaction-State: `is_transaction_active()`, `is_rollback_only()`, `set_rollback_only()`
  - `TransactionGuard` für RAII-Style Transaction-Management
  - Drop-Guard mit Warning bei offenen Transactions (tracing Feature)
  - 10 neue Unit-Tests für Transaction-Funktionalität (65 Tests gesamt)

### Changed
- **BREAKING**: Projekt von Multi-Crate Workspace zu Monolith-Crate umstrukturiert
- Crate umbenannt von `rustine` zu `rustine-dbal`
- Module: `rustine-core` → `core/`, `rustine-driver` → `driver/`, etc.
- Imports ändern sich: `use rustine_dbal::core::*` statt `use rustine_core::*`

## [0.1.0] - 2024-12-02

### Added

#### Core Foundation (Epic 1)
- **core**: Error-Hierarchie (`Error`, `ConnectionError`, `TransactionError`, `QueryError`, `SchemaError`)
- **core**: `SqlValue` enum mit 15+ Varianten für alle SQL-Typen
- **core**: `ToSql` trait für Rust → SQL Konvertierung
- **core**: `FromSql` trait für SQL → Rust Konvertierung
- **core**: `ParameterType` enum für Prepared Statement Binding
- **core**: `ConnectionParams` für Verbindungskonfiguration
- **core**: `Configuration` für Runtime-Einstellungen
- **core**: `IsolationLevel` enum für Transaction Isolation
- **core**: Feature-Flags für `chrono`, `uuid`, `json`, `decimal`

#### Driver Abstraktion
- **driver**: `Driver` trait für Datenbank-Treiber
- **driver**: `DriverConnection` trait für Verbindungen
- **driver**: `DriverStatement` trait für Prepared Statements
- **driver**: `DriverResult` trait für Query-Ergebnisse

#### Platform Abstraktion
- **platform**: `Platform` trait für SQL-Dialekte
- **platform**: `PostgresPlatform` Grundstruktur
- **platform**: `MySqlPlatform` Grundstruktur
- **platform**: `SqlitePlatform` Grundstruktur

#### Dokumentation
- PRD (Product Requirements Document) nach BMAD Method v6
- Architecture Decision Document mit 7 ADRs
- Epic Breakdown mit 6 Epics und 29 Stories
- Doctrine DBAL Analyse-Dokumentation (8 Dokumente)
- Conventional Commits Richtlinien
- SemVer Dokumentation in Cargo.toml

#### Tests
- 43 Unit Tests (38 in core, 5 in platform)
- 3 Doc-Tests

### Infrastructure
- Monolith-Crate Struktur (`rustine-dbal`)
- GitHub Repository eingerichtet
- .gitignore für Rust-Projekte

[Unreleased]: https://github.com/elmar-roeser/rustine-dbal/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/elmar-roeser/rustine-dbal/releases/tag/v0.1.0
