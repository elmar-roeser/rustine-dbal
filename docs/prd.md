# Product Requirements Document - Rustine

**Author:** Elmar
**Date:** 2024-12-02
**Version:** 1.0
**Based on:** BMAD Method v6

---

## Executive Summary

### Vision

**Rustine** ist eine idiomatische Rust-Implementierung der Doctrine DBAL (Database Abstraction Layer) Konzepte. Das Ziel ist es, Rust-Entwicklern eine vertraute, mächtige Datenbank-Abstraktionsschicht zu bieten, die die bewährte Architektur von Doctrine DBAL nutzt, aber vollständig auf moderne Rust-Patterns und das Rust-Ökosystem zugeschnitten ist.

### Was Rustine besonders macht

1. **Bewährte Architektur**: Doctrine DBAL ist seit über 15 Jahren battle-tested in Millionen von PHP-Projekten. Diese Erfahrung fließt in Rustine ein.

2. **Idiomatisches Rust**: Keine 1:1-Portierung, sondern eine Neuimplementierung die Rust-Stärken nutzt:
   - Ownership & Borrowing statt GC
   - Traits statt Vererbung
   - Result<T,E> statt Exceptions
   - Async-first Design
   - Zero-cost Abstractions

3. **Lücke im Rust-Ökosystem**: Es gibt keine vollständige DBAL in Rust. `sqlx` ist exzellent für direktes SQL, aber bietet keine Schema-Abstraktion oder plattformunabhängige DDL-Generierung.

4. **ORM-Foundation**: Rustine ist der Grundstein für ein zukünftiges Doctrine ORM Äquivalent in Rust.

### Projekt-Klassifikation

| Eigenschaft | Wert |
|-------------|------|
| **Projekt-Typ** | Library / Framework |
| **Domain** | Database / Infrastructure |
| **Komplexität** | Hoch |
| **Sprache** | Rust |
| **Lizenz** | MIT / Apache-2.0 (dual) |

---

## Success Criteria

### Messbare Ziele

| Kriterium | Ziel | Messung |
|-----------|------|---------|
| **API Coverage** | 80% der Doctrine DBAL Public API | Feature-Checklist |
| **Platform Support** | PostgreSQL, MySQL, SQLite | Integration Tests |
| **Performance** | ≤10% Overhead vs raw sqlx | Benchmarks |
| **Documentation** | 100% public API dokumentiert | rustdoc coverage |
| **Test Coverage** | ≥80% Line Coverage | cargo tarpaulin |

### Definition of Done (MVP)

Das MVP ist erreicht wenn:

- [ ] Connection-Management mit Lazy-Connect funktioniert
- [ ] Prepared Statements mit Parameter-Binding funktionieren
- [ ] Transactions mit Savepoint-Nesting funktionieren
- [ ] QueryBuilder SELECT/INSERT/UPDATE/DELETE generiert korrektes SQL
- [ ] Schema-Introspection für PostgreSQL funktioniert
- [ ] Mindestens 3 Datenbanken unterstützt werden (PostgreSQL, SQLite, MySQL)
- [ ] Crate auf crates.io veröffentlicht ist

---

## Scope Definition

### MVP Scope (v0.1)

**Enthalten:**
- Connection & Configuration
- Prepared Statements
- Transaction Management (inkl. Savepoints)
- Basic QueryBuilder (SELECT, INSERT, UPDATE, DELETE)
- Platform Abstraction (PostgreSQL, SQLite, MySQL)
- Schema Introspection (Tables, Columns, Indexes)
- Type System (Basic Types)

**Nicht enthalten:**
- Schema Migration Generation
- Query Caching
- Connection Pooling (nutzt sqlx built-in)
- ORM Features

### Growth Scope (v0.2 - v0.5)

- Schema Diff & Migration SQL Generation
- Vollständige Type Registry mit Custom Types
- UNION, CTE (WITH), Subqueries im QueryBuilder
- Alle Schema-Objekte (Views, Sequences, Foreign Keys)
- Performance Optimierungen
- SQL Server Support

### Vision Scope (v1.0+)

- Proc-Macro für Type-Definitionen
- Event/Middleware System
- Query Result Caching
- Vollständige Doctrine DBAL Feature-Parität
- Basis für Rustine ORM

---

## Developer Journeys

### Journey 1: Erste Datenbankverbindung

**Persona:** Rust-Entwickler, neu bei Rustine

**Szenario:** Entwickler möchte eine PostgreSQL-Datenbank verbinden und erste Queries ausführen.

```rust
// Was der Entwickler erwartet:
let conn = Connection::new("postgres://user:pass@localhost/db").await?;
let rows = conn.fetch_all("SELECT * FROM users WHERE active = $1", &[&true]).await?;
```

**Erfolgskriterien:**
- Verbindung in ≤3 Zeilen Code
- Klare Fehlermeldungen bei falschen Credentials
- Dokumentation mit Beispiel auf docs.rs

### Journey 2: Query Builder nutzen

**Persona:** Entwickler die dynamische Queries bauen

**Szenario:** Entwickler baut eine Suchabfrage mit optionalen Filtern.

```rust
let mut qb = conn.query_builder()
    .select(&["id", "name", "email"])
    .from("users", Some("u"));

if let Some(name) = search_name {
    qb = qb.and_where(expr::like("u.name", ":name"))
           .set_parameter("name", format!("%{}%", name));
}

let users = qb.fetch_all().await?;
```

**Erfolgskriterien:**
- Fluent API mit Method Chaining
- Compile-Time Fehler bei falscher Verwendung wo möglich
- SQL-Injection sicher by default

### Journey 3: Schema-Introspection

**Persona:** Tool-Entwickler (z.B. Code-Generator)

**Szenario:** Entwickler möchte Tabellen-Struktur auslesen.

```rust
let schema_manager = conn.schema_manager();
let tables = schema_manager.list_tables().await?;

for table in &tables {
    println!("Table: {}", table.name());
    for column in table.columns() {
        println!("  - {}: {:?}", column.name(), column.column_type());
    }
}
```

**Erfolgskriterien:**
- Vollständige Tabellen-Information abrufbar
- Platform-unabhängige Column-Types
- Async-friendly API

### Journey 4: Transaction Handling

**Persona:** Entwickler mit Business-Logik

**Szenario:** Entwickler führt mehrere Operationen atomar aus.

```rust
conn.transactional(|tx| async move {
    let user_id = tx.insert("users", &[("name", &name)]).await?;
    tx.insert("profiles", &[("user_id", &user_id)]).await?;
    Ok(user_id)
}).await?;
```

**Erfolgskriterien:**
- Automatisches Rollback bei Fehler
- Savepoint-Support für Nested Transactions
- Keine vergessenen offenen Transactions (RAII)

---

## Technical Domain Requirements

### Database-spezifische Anforderungen

| Requirement | Beschreibung |
|-------------|--------------|
| **SQL-Dialekte** | Korrekte Generierung für PostgreSQL, MySQL, SQLite |
| **Identifier Quoting** | Plattform-korrekt (`"`, `` ` ``, `[]`) |
| **Type Mapping** | Bidirektionale Konvertierung Rust ↔ DB |
| **NULL Handling** | Korrektes Mapping zu `Option<T>` |
| **Transaction Isolation** | Alle Standard-Level unterstützt |
| **Prepared Statements** | Parameter-Binding für alle Plattformen |

### Compliance & Standards

- SQL-92 als Baseline
- Platform-spezifische Erweiterungen dokumentiert
- Keine SQL-Injection Möglichkeiten in der API

---

## Functional Requirements

### Core - Connection Management

- **FR1:** Entwickler können eine Connection mit Connection-String erstellen
- **FR2:** Entwickler können eine Connection mit Builder-Pattern konfigurieren
- **FR3:** Connection unterstützt Lazy-Connect (erste Query triggert Verbindung)
- **FR4:** Entwickler können Verbindung explizit schließen
- **FR5:** Entwickler können Verbindungsstatus abfragen

### Core - Query Execution

- **FR6:** Entwickler können SQL-Queries mit Parametern ausführen
- **FR7:** Entwickler können Prepared Statements erstellen und wiederverwenden
- **FR8:** System unterstützt Named Parameters (`:name`) und Positional Parameters (`$1`, `?`)
- **FR9:** Entwickler können einzelne Zeilen fetchen
- **FR10:** Entwickler können alle Zeilen fetchen
- **FR11:** Entwickler können Ergebnisse als Stream iterieren
- **FR12:** Entwickler können affected rows bei INSERT/UPDATE/DELETE abrufen

### Core - Transactions

- **FR13:** Entwickler können Transactions explizit starten/committen/rollbacken
- **FR14:** System unterstützt Nested Transactions via Savepoints
- **FR15:** Entwickler können Transactions via Closure (auto-commit/rollback) nutzen
- **FR16:** System verhindert vergessene offene Transactions (Warnung/Rollback bei Drop)
- **FR17:** Entwickler können Transaction Isolation Level setzen

### QueryBuilder

- **FR18:** Entwickler können SELECT-Queries programmatisch bauen
- **FR19:** Entwickler können INSERT-Queries programmatisch bauen
- **FR20:** Entwickler können UPDATE-Queries programmatisch bauen
- **FR21:** Entwickler können DELETE-Queries programmatisch bauen
- **FR22:** QueryBuilder unterstützt JOINs (INNER, LEFT, RIGHT)
- **FR23:** QueryBuilder unterstützt WHERE mit AND/OR Kombinationen
- **FR24:** QueryBuilder unterstützt ORDER BY, GROUP BY, HAVING
- **FR25:** QueryBuilder unterstützt LIMIT/OFFSET
- **FR26:** QueryBuilder hat ExpressionBuilder für Conditions
- **FR27:** QueryBuilder integriert Parameter-Binding automatisch

### Platform Abstraction

- **FR28:** System generiert plattform-korrektes SQL
- **FR29:** System abstrahiert Identifier-Quoting pro Plattform
- **FR30:** System abstrahiert LIMIT/OFFSET Syntax pro Plattform
- **FR31:** System mappt Rust-Types auf plattform-korrekte SQL-Types
- **FR32:** System erkennt Datenbankversion und wählt passende Platform

### Schema Management

- **FR33:** Entwickler können Liste aller Tabellen abrufen
- **FR34:** Entwickler können Spalten einer Tabelle introspizieren
- **FR35:** Entwickler können Indexes einer Tabelle abrufen
- **FR36:** Entwickler können Foreign Keys abrufen
- **FR37:** Entwickler können vollständiges Schema als Objekt-Graph abrufen
- **FR38:** Entwickler können prüfen ob Tabelle existiert

### Type System

- **FR39:** System konvertiert automatisch zwischen Rust-Types und DB-Values
- **FR40:** Entwickler können Custom Types registrieren
- **FR41:** System unterstützt alle gängigen Rust-Typen (i32, i64, String, bool, etc.)
- **FR42:** System unterstützt DateTime-Types (chrono)
- **FR43:** System unterstützt JSON-Types (serde_json)
- **FR44:** System unterstützt UUID-Types
- **FR45:** System unterstützt Decimal-Types (rust_decimal)

### Error Handling

- **FR46:** System liefert strukturierte Fehler mit Kontext
- **FR47:** Fehler sind in Kategorien unterteilt (Connection, Query, Schema, etc.)
- **FR48:** Constraint-Violations sind als spezifische Fehler erkennbar
- **FR49:** Fehler enthalten SQL und Parameter für Debugging (redacted für Secrets)

---

## Non-Functional Requirements

### Performance

| NFR | Requirement | Messung |
|-----|-------------|---------|
| **NFR1** | Query-Overhead ≤10% vs raw sqlx | Benchmark |
| **NFR2** | Connection-Aufbau <100ms | Benchmark |
| **NFR3** | Zero-Copy wo möglich | Code Review |
| **NFR4** | Keine unnötigen Allocations im Hot Path | Profiling |

### Reliability

| NFR | Requirement | Messung |
|-----|-------------|---------|
| **NFR5** | Keine Panics in Public API | Test Suite |
| **NFR6** | Thread-safe (Send + Sync wo sinnvoll) | Compile-Time |
| **NFR7** | Graceful Handling von Connection-Verlust | Integration Tests |

### Usability

| NFR | Requirement | Messung |
|-----|-------------|---------|
| **NFR8** | Intuitive API für Doctrine-Kenner | User Feedback |
| **NFR9** | Hilfreiche Compile-Time Errors | User Feedback |
| **NFR10** | Vollständige rustdoc Dokumentation | Doc Coverage |
| **NFR11** | Beispiele für alle Hauptfeatures | Doc Review |

### Maintainability

| NFR | Requirement | Messung |
|-----|-------------|---------|
| **NFR12** | Modulare Crate-Struktur | Architecture Review |
| **NFR13** | ≥80% Test Coverage | cargo tarpaulin |
| **NFR14** | CI/CD Pipeline | GitHub Actions |
| **NFR15** | Semantic Versioning | Release Process |

### Compatibility

| NFR | Requirement | Messung |
|-----|-------------|---------|
| **NFR16** | Rust Edition 2021 | Cargo.toml |
| **NFR17** | MSRV dokumentiert | README |
| **NFR18** | Kompatibel mit Tokio Runtime | Integration Tests |
| **NFR19** | Optional async-std Support | Feature Flag |

---

## Appendix

### Referenz-Dokumentation

Die detaillierte Analyse der Doctrine DBAL Architektur befindet sich in:

- [docs/00-overview.md](00-overview.md) - Architektur-Übersicht
- [docs/01-connection.md](01-connection.md) - Connection-Komponente
- [docs/02-driver.md](02-driver.md) - Driver-Layer
- [docs/03-platform.md](03-platform.md) - Platform/SQL-Dialekte
- [docs/04-query-builder.md](04-query-builder.md) - QueryBuilder
- [docs/05-schema.md](05-schema.md) - Schema-Management
- [docs/06-types.md](06-types.md) - Type-System
- [docs/07-rust-mapping.md](07-rust-mapping.md) - Rust-Portierungs-Strategie

### Nächste Schritte

Nach Abschluss des PRD:

1. **Architecture Document** - Detaillierte technische Architektur
2. **Epic Breakdown** - FRs in implementierbare Epics/Stories aufteilen
3. **Implementation** - Coding beginnen

---

*Dieses PRD basiert auf der BMAD Method v6 und dient als Grundlage für alle weiteren Entwicklungsarbeiten an Rustine.*
