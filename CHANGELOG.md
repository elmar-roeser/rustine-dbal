# Changelog

Alle wichtigen Änderungen an diesem Projekt werden in dieser Datei dokumentiert.

Das Format basiert auf [Keep a Changelog](https://keepachangelog.com/de/1.1.0/),
und dieses Projekt folgt [Semantic Versioning](https://semver.org/lang/de/).

## [Unreleased]

### Added
- Nächste Features kommen hier...

## [0.1.0] - 2024-12-02

### Added

#### Core Foundation (Epic 1)
- **rustine-core**: Error-Hierarchie (`Error`, `ConnectionError`, `TransactionError`, `QueryError`, `SchemaError`)
- **rustine-core**: `SqlValue` enum mit 15+ Varianten für alle SQL-Typen
- **rustine-core**: `ToSql` trait für Rust → SQL Konvertierung
- **rustine-core**: `FromSql` trait für SQL → Rust Konvertierung
- **rustine-core**: `ParameterType` enum für Prepared Statement Binding
- **rustine-core**: `ConnectionParams` für Verbindungskonfiguration
- **rustine-core**: `Configuration` für Runtime-Einstellungen
- **rustine-core**: `IsolationLevel` enum für Transaction Isolation
- **rustine-core**: Feature-Flags für `chrono`, `uuid`, `json`, `decimal`

#### Driver Abstraktion
- **rustine-driver**: `Driver` trait für Datenbank-Treiber
- **rustine-driver**: `DriverConnection` trait für Verbindungen
- **rustine-driver**: `DriverStatement` trait für Prepared Statements
- **rustine-driver**: `DriverResult` trait für Query-Ergebnisse

#### Platform Abstraktion
- **rustine-platform**: `Platform` trait für SQL-Dialekte
- **rustine-platform**: `PostgresPlatform` Grundstruktur
- **rustine-platform**: `MySqlPlatform` Grundstruktur
- **rustine-platform**: `SqlitePlatform` Grundstruktur

#### Dokumentation
- PRD (Product Requirements Document) nach BMAD Method v6
- Architecture Decision Document mit 7 ADRs
- Epic Breakdown mit 6 Epics und 29 Stories
- Doctrine DBAL Analyse-Dokumentation (8 Dokumente)
- Conventional Commits Richtlinien

#### Tests
- 43 Unit Tests (38 in rustine-core, 5 in rustine-platform)
- 2 Doc-Tests

### Infrastructure
- Workspace-Struktur mit 6 Crates
- GitHub Repository eingerichtet
- .gitignore für Rust-Projekte

[Unreleased]: https://github.com/elmar-roeser/rustine-dbal/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/elmar-roeser/rustine-dbal/releases/tag/v0.1.0
