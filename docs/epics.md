# Rustine - Epic Breakdown

**Author:** Elmar
**Date:** 2024-12-02
**Based on:** BMAD Method v6

---

## Overview

Dieses Dokument zerlegt die 49 Functional Requirements aus dem [PRD](prd.md) in implementierbare Epics und Stories, basierend auf den technischen Entscheidungen aus dem [Architecture Document](architecture.md).

**Epics Summary:**

| Epic | Titel | Stories | FRs |
|------|-------|---------|-----|
| 1 | Core Foundation | 6 | FR39-45, FR46-49 |
| 2 | Database Connectivity | 5 | FR1-5, FR6-12 |
| 3 | Transaction Management | 4 | FR13-17 |
| 4 | Platform Abstraction | 4 | FR28-32 |
| 5 | Query Builder | 6 | FR18-27 |
| 6 | Schema Introspection | 4 | FR33-38 |

**Total:** 6 Epics, 29 Stories, 49 FRs

---

## Functional Requirements Inventory

### Core (FR39-49)
- FR39: Automatische Type-Konvertierung Rust ↔ DB
- FR40: Custom Types registrierbar
- FR41-45: Standard-Type-Support (primitives, DateTime, JSON, UUID, Decimal)
- FR46: Strukturierte Fehler mit Kontext
- FR47: Kategorisierte Fehler
- FR48: Constraint-Violations erkennbar
- FR49: Fehler enthalten SQL (redacted)

### Connection (FR1-12)
- FR1-5: Connection Management (String, Builder, Lazy, Close, Status)
- FR6-12: Query Execution (Params, Prepared, Named/Positional, Fetch variants, Affected rows)

### Transactions (FR13-17)
- FR13: Explicit begin/commit/rollback
- FR14: Nested Transactions via Savepoints
- FR15: Closure-based transactions
- FR16: Drop-Guard für offene Transactions
- FR17: Transaction Isolation Level

### Platform (FR28-32)
- FR28: Platform-korrektes SQL
- FR29: Identifier-Quoting
- FR30: LIMIT/OFFSET Syntax
- FR31: Type-Mapping
- FR32: Version-Detection

### QueryBuilder (FR18-27)
- FR18-21: CRUD Queries (SELECT, INSERT, UPDATE, DELETE)
- FR22-25: Query Features (JOINs, WHERE, ORDER BY, LIMIT)
- FR26: ExpressionBuilder
- FR27: Parameter-Binding Integration

### Schema (FR33-38)
- FR33: List Tables
- FR34: Introspect Columns
- FR35: Introspect Indexes
- FR36: Introspect Foreign Keys
- FR37: Full Schema as Object
- FR38: Table Exists Check

---

## FR Coverage Map

```
Epic 1: Core Foundation
├── FR39 → Story 1.3 (ToSql/FromSql)
├── FR40 → Story 1.4 (TypeRegistry)
├── FR41 → Story 1.3 (Primitives)
├── FR42 → Story 1.5 (DateTime)
├── FR43 → Story 1.5 (JSON)
├── FR44 → Story 1.5 (UUID)
├── FR45 → Story 1.5 (Decimal)
├── FR46 → Story 1.1 (Error types)
├── FR47 → Story 1.1 (Error categories)
├── FR48 → Story 1.2 (Constraint errors)
└── FR49 → Story 1.2 (SQL in errors)

Epic 2: Database Connectivity
├── FR1  → Story 2.1 (Connection string)
├── FR2  → Story 2.1 (Builder)
├── FR3  → Story 2.2 (Lazy connect)
├── FR4  → Story 2.2 (Close)
├── FR5  → Story 2.2 (Status)
├── FR6  → Story 2.3 (Execute with params)
├── FR7  → Story 2.4 (Prepared statements)
├── FR8  → Story 2.3 (Named/Positional)
├── FR9  → Story 2.5 (Fetch one)
├── FR10 → Story 2.5 (Fetch all)
├── FR11 → Story 2.5 (Stream)
└── FR12 → Story 2.3 (Affected rows)

Epic 3: Transaction Management
├── FR13 → Story 3.1 (Begin/Commit/Rollback)
├── FR14 → Story 3.2 (Savepoints)
├── FR15 → Story 3.3 (Closure)
├── FR16 → Story 3.4 (Drop guard)
└── FR17 → Story 3.1 (Isolation level)

Epic 4: Platform Abstraction
├── FR28 → Story 4.2, 4.3, 4.4 (Platform SQL)
├── FR29 → Story 4.1 (Quoting)
├── FR30 → Story 4.1 (LIMIT/OFFSET)
├── FR31 → Story 4.1 (Type mapping)
└── FR32 → Story 4.2, 4.3, 4.4 (Version detection)

Epic 5: Query Builder
├── FR18 → Story 5.1 (SELECT)
├── FR19 → Story 5.2 (INSERT)
├── FR20 → Story 5.3 (UPDATE)
├── FR21 → Story 5.4 (DELETE)
├── FR22 → Story 5.5 (JOINs)
├── FR23 → Story 5.1 (WHERE)
├── FR24 → Story 5.1 (ORDER BY, GROUP BY)
├── FR25 → Story 5.1 (LIMIT/OFFSET)
├── FR26 → Story 5.6 (ExpressionBuilder)
└── FR27 → Story 5.1-5.4 (Params)

Epic 6: Schema Introspection
├── FR33 → Story 6.1 (List tables)
├── FR34 → Story 6.2 (Columns)
├── FR35 → Story 6.3 (Indexes)
├── FR36 → Story 6.3 (Foreign keys)
├── FR37 → Story 6.4 (Full schema)
└── FR38 → Story 6.1 (Table exists)
```

---

## Epic 1: Core Foundation

**Goal:** Etabliere die Basis-Infrastruktur (Types, Errors) die alle anderen Crates benötigen.

**Crate:** `rustine-core`

**Warum zuerst:** Alle anderen Epics hängen von Error-Types und SqlValue ab.

---

### Story 1.1: Error Type Hierarchy

Als **Library-Nutzer**,
möchte ich **strukturierte, kategorisierte Fehler erhalten**,
damit ich **gezielt auf verschiedene Fehlersituationen reagieren kann**.

**Acceptance Criteria:**

**Given** eine Datenbankoperation schlägt fehl
**When** ich den Fehler erhalte
**Then** kann ich per Pattern-Matching die Fehlerkategorie bestimmen

**And** der Fehler enthält eine aussagekräftige Nachricht
**And** der Fehler implementiert `std::error::Error`
**And** der Fehler ist `Send + Sync`

**Technical Notes:**
- Implementiere `Error` enum in `rustine-core/src/error.rs`
- Nutze `thiserror` für Derivation
- Sub-Enums: `ConnectionError`, `TransactionError`, `SchemaError`, `ConversionError`

**FRs:** FR46, FR47

---

### Story 1.2: Error Context & SQL Inclusion

Als **Library-Nutzer**,
möchte ich **Fehler mit Kontext wie SQL-Query und Constraint-Namen erhalten**,
damit ich **Probleme schnell debuggen kann**.

**Acceptance Criteria:**

**Given** eine Query fehlschlägt
**When** ich den Query-Fehler erhalte
**Then** enthält er optional das ausgeführte SQL

**Given** ein Unique-Constraint verletzt wird
**When** ich den Fehler erhalte
**Then** kann ich ihn als `ConstraintViolation` identifizieren

**And** Passwörter/Secrets in Connection-Strings werden redacted

**Technical Notes:**
- `QueryError` struct mit `sql: Option<String>` und `params: Option<String>`
- `ConstraintViolation` als Error-Variant
- Implement `redact_credentials()` für Connection-Strings

**FRs:** FR48, FR49

---

### Story 1.3: SqlValue & Basic Type Traits

Als **Library-Nutzer**,
möchte ich **Rust-Primitive automatisch zu SQL-Werten konvertieren**,
damit ich **typsicher mit der Datenbank arbeiten kann**.

**Acceptance Criteria:**

**Given** ich habe einen `i32` Wert
**When** ich ihn als Query-Parameter übergebe
**Then** wird er automatisch zu `SqlValue::I32` konvertiert

**And** alle Rust-Primitiven sind unterstützt: `bool`, `i8-i64`, `f32`, `f64`, `String`, `&str`, `Vec<u8>`
**And** `Option<T>` wird zu `SqlValue::Null` wenn `None`
**And** bidirektionale Konvertierung funktioniert (ToSql + FromSql)

**Technical Notes:**
- `SqlValue` enum in `rustine-core/src/types.rs`
- `ToSql` trait mit `fn to_sql(&self) -> SqlValue`
- `FromSql` trait mit `fn from_sql(value: SqlValue) -> Result<Self, ConversionError>`
- Blanket implementations für Primitiven

**FRs:** FR39, FR41

---

### Story 1.4: Type Registry für Custom Types

Als **Library-Nutzer**,
möchte ich **eigene Types registrieren können**,
damit ich **Domain-spezifische Typen (z.B. Money) verwenden kann**.

**Acceptance Criteria:**

**Given** ich habe einen Custom-Type `Money`
**When** ich ihn in der TypeRegistry registriere
**Then** kann ich ihn in Queries verwenden

**And** die Registry ist thread-safe (RwLock)
**And** builtin Types sind vorregistriert

**Technical Notes:**
- `TypeRegistry` struct mit `HashMap<TypeId, Box<dyn SqlType>>`
- `SqlType` trait für dynamische Type-Registrierung
- Global Registry via `lazy_static` oder `OnceLock`

**FRs:** FR40

---

### Story 1.5: Extended Type Support

Als **Library-Nutzer**,
möchte ich **DateTime, JSON, UUID und Decimal Types verwenden**,
damit ich **reale Anwendungsdaten speichern kann**.

**Acceptance Criteria:**

**Given** ich habe ein `chrono::NaiveDateTime`
**When** ich es als Parameter übergebe
**Then** wird es korrekt zu SQL konvertiert

**And** `chrono::{NaiveDate, NaiveTime, NaiveDateTime, DateTime<Utc>}` sind unterstützt
**And** `serde_json::Value` ist unterstützt
**And** `uuid::Uuid` ist unterstützt
**And** `rust_decimal::Decimal` ist unterstützt

**Technical Notes:**
- Feature-Flags: `chrono`, `json`, `uuid`, `decimal`
- Implementiere `ToSql`/`FromSql` für jeden Typ
- Date-Format ist Platform-abhängig (über Platform-Trait)

**FRs:** FR42, FR43, FR44, FR45

---

### Story 1.6: Row & Parameters Types

Als **Library-Nutzer**,
möchte ich **Query-Ergebnisse als typisierte Rows erhalten**,
damit ich **einfach auf Spalten zugreifen kann**.

**Acceptance Criteria:**

**Given** eine Query gibt Ergebnisse zurück
**When** ich eine Row erhalte
**Then** kann ich Spalten per Name oder Index abrufen

**And** `row.get::<T>("column")` gibt typisiertes Result zurück
**And** `row.try_get::<T>("column")` gibt `Option<T>` für nullable Spalten
**And** Parameters können per Named (`:name`) oder Positional (`$1`) gebunden werden

**Technical Notes:**
- `Row` struct mit `columns: IndexMap<String, SqlValue>`
- Generic `get<T: FromSql>(&self, column: &str) -> Result<T, Error>`
- `Parameters` struct für Query-Parameter

**FRs:** Unterstützt FR6-FR12

---

## Epic 2: Database Connectivity

**Goal:** Ermögliche Verbindung zu Datenbanken und Ausführung von Queries.

**Crates:** `rustine-driver`, `rustine` (Connection)

**Prerequisites:** Epic 1 (Core Foundation)

---

### Story 2.1: Connection Creation

Als **Library-Nutzer**,
möchte ich **eine Datenbankverbindung mit Connection-String oder Builder erstellen**,
damit ich **flexibel verbinden kann**.

**Acceptance Criteria:**

**Given** ich habe einen Connection-String `postgres://user:pass@localhost/db`
**When** ich `Connection::new(url).await` aufrufe
**Then** erhalte ich eine verbundene Connection

**Given** ich nutze den Builder
**When** ich `Connection::builder().host("localhost").database("db").build().await` aufrufe
**Then** erhalte ich eine verbundene Connection

**And** ungültige Connection-Strings geben `ConnectionError::InvalidUrl`
**And** fehlgeschlagene Verbindungen geben `ConnectionError::Refused`

**Technical Notes:**
- `Connection<D: Driver>` struct
- `ConnectionBuilder` mit fluent API
- URL-Parsing für Connection-String

**FRs:** FR1, FR2

---

### Story 2.2: Lazy Connection & State Management

Als **Library-Nutzer**,
möchte ich **dass die Verbindung erst bei erster Query hergestellt wird**,
damit ich **Connections vorab konfigurieren kann ohne sofort zu verbinden**.

**Acceptance Criteria:**

**Given** ich erstelle eine Connection
**When** ich noch keine Query ausgeführt habe
**Then** ist `conn.is_connected()` false

**Given** ich führe die erste Query aus
**When** die Connection noch nicht verbunden war
**Then** wird automatisch verbunden (lazy)

**And** `conn.close()` trennt die Verbindung
**And** nach `close()` ist `is_connected()` false

**Technical Notes:**
- `inner: Option<D::Connection>` für lazy state
- `ensure_connected()` interne Methode
- `is_connected()` und `close()` public API

**FRs:** FR3, FR4, FR5

---

### Story 2.3: Query Execution with Parameters

Als **Library-Nutzer**,
möchte ich **SQL-Queries mit Parametern ausführen**,
damit ich **sicher gegen SQL-Injection bin**.

**Acceptance Criteria:**

**Given** ich habe eine Query mit Parametern
**When** ich `conn.execute("INSERT INTO users (name) VALUES ($1)", &[&name]).await` aufrufe
**Then** werden die Parameter sicher gebunden

**And** Named Parameters (`:name`) werden unterstützt
**And** Positional Parameters (`$1`, `?`) werden unterstützt
**And** `execute_statement()` gibt affected rows als `u64` zurück

**Technical Notes:**
- `execute_query()` für SELECT (returns Stream/Vec)
- `execute_statement()` für INSERT/UPDATE/DELETE (returns u64)
- Parameter-Binding über `&[&dyn ToSql]`

**FRs:** FR6, FR8, FR12

---

### Story 2.4: Prepared Statements

Als **Library-Nutzer**,
möchte ich **Prepared Statements erstellen und wiederverwenden**,
damit ich **Performance bei wiederholten Queries optimieren kann**.

**Acceptance Criteria:**

**Given** ich habe eine Query die oft ausgeführt wird
**When** ich `conn.prepare("SELECT * FROM users WHERE id = $1").await` aufrufe
**Then** erhalte ich ein `Statement` das ich wiederverwenden kann

**And** `stmt.execute(&[&id]).await` führt das Statement aus
**And** Statement ist mit Connection-Lifetime verbunden

**Technical Notes:**
- `Statement<'conn>` struct
- `prepare()` auf Connection
- Lifetime-bound an Connection

**FRs:** FR7

---

### Story 2.5: Fetch Methods

Als **Library-Nutzer**,
möchte ich **verschiedene Fetch-Methoden für Query-Ergebnisse**,
damit ich **die passende Methode für meinen Use-Case nutzen kann**.

**Acceptance Criteria:**

**Given** eine SELECT-Query
**When** ich `fetch_one()` aufrufe
**Then** erhalte ich `Option<Row>` (None wenn keine Ergebnisse)

**When** ich `fetch_all()` aufrufe
**Then** erhalte ich `Vec<Row>`

**When** ich `fetch()` aufrufe
**Then** erhalte ich einen `Stream<Item = Result<Row, Error>>`

**And** `fetch_optional()` gibt `Option<Row>` ohne Error bei 0 Ergebnissen
**And** `fetch_scalar::<T>()` gibt ersten Wert der ersten Zeile

**Technical Notes:**
- Verschiedene Fetch-Methoden auf Connection
- `impl Stream` für lazy iteration
- Generics für typisiertes Fetching

**FRs:** FR9, FR10, FR11

---

## Epic 3: Transaction Management

**Goal:** Ermögliche transaktionale Operationen mit Savepoint-Support.

**Crate:** `rustine-driver` (Teil von Connection)

**Prerequisites:** Epic 2 (Database Connectivity)

---

### Story 3.1: Basic Transaction Control

Als **Library-Nutzer**,
möchte ich **Transactions explizit starten, committen und rollbacken**,
damit ich **atomare Operationen durchführen kann**.

**Acceptance Criteria:**

**Given** ich habe eine aktive Connection
**When** ich `conn.begin_transaction().await` aufrufe
**Then** wird eine Transaction gestartet

**When** ich `conn.commit().await` aufrufe
**Then** werden alle Änderungen persistiert

**When** ich `conn.rollback().await` aufrufe
**Then** werden alle Änderungen verworfen

**And** `set_transaction_isolation(level)` setzt das Isolation-Level vor begin
**And** Isolation-Levels: ReadUncommitted, ReadCommitted, RepeatableRead, Serializable

**Technical Notes:**
- `begin_transaction()`, `commit()`, `rollback()` auf Connection
- `TransactionIsolationLevel` enum
- State-Tracking via `transaction_nesting: AtomicU32`

**FRs:** FR13, FR17

---

### Story 3.2: Nested Transactions via Savepoints

Als **Library-Nutzer**,
möchte ich **verschachtelte Transactions über Savepoints**,
damit ich **Teilbereiche unabhängig rollbacken kann**.

**Acceptance Criteria:**

**Given** ich bin in einer Transaction
**When** ich `begin_transaction()` erneut aufrufe
**Then** wird ein Savepoint erstellt (nicht echte nested TX)

**When** ich `commit()` auf Level 2 aufrufe
**Then** wird der Savepoint released

**When** ich `rollback()` auf Level 2 aufrufe
**Then** wird nur bis zum Savepoint zurückgerollt

**And** Savepoint-Namen sind `RUSTINE_1`, `RUSTINE_2`, etc.
**And** `transaction_nesting_level()` gibt aktuelle Tiefe zurück

**Technical Notes:**
- Savepoint-SQL über Platform-Trait
- Nesting-Counter in Connection
- MySQL: `supports_release_savepoints() = false`

**FRs:** FR14

---

### Story 3.3: Transactional Closure

Als **Library-Nutzer**,
möchte ich **eine Closure in einer Transaction ausführen mit auto-commit/rollback**,
damit ich **Transaction-Handling nicht vergessen kann**.

**Acceptance Criteria:**

**Given** ich habe eine Closure die DB-Operationen macht
**When** ich `conn.transactional(|tx| async { ... }).await` aufrufe
**Then** wird bei `Ok` committed, bei `Err` rollbacked

**And** der Rückgabewert der Closure wird durchgereicht
**And** Panics in der Closure führen zu Rollback

**Technical Notes:**
- `transactional<F, T>(f: F) -> Result<T, Error>` generic method
- `async move` Closure
- `catch_unwind` für Panic-Safety (optional)

**FRs:** FR15

---

### Story 3.4: Transaction Drop Guard

Als **Library-Nutzer**,
möchte ich **gewarnt werden wenn eine Transaction nicht abgeschlossen wird**,
damit ich **keine offenen Transactions vergesse**.

**Acceptance Criteria:**

**Given** eine Transaction ist aktiv
**When** die Connection gedroppt wird
**Then** wird ein Warning geloggt (via tracing wenn enabled)

**And** die Transaction wird automatisch rollbacked
**And** `is_transaction_active()` zeigt ob TX offen ist
**And** `is_rollback_only()` zeigt ob TX nur noch rollback erlaubt

**Technical Notes:**
- `Drop` impl für Connection mit TX-Check
- `is_rollback_only: AtomicBool` state
- Optional: `tracing::warn!` wenn Feature enabled

**FRs:** FR16

---

## Epic 4: Platform Abstraction

**Goal:** Abstrahiere SQL-Dialekt-Unterschiede zwischen Datenbanken.

**Crate:** `rustine-platform`

**Prerequisites:** Epic 1 (Core Foundation)

---

### Story 4.1: Platform Trait & Common SQL

Als **Library-Entwickler**,
möchte ich **einen Platform-Trait der SQL-Unterschiede abstrahiert**,
damit ich **platformunabhängig SQL generieren kann**.

**Acceptance Criteria:**

**Given** ich brauche plattform-spezifisches SQL
**When** ich `platform.quote_identifier("user")` aufrufe
**Then** erhalte ich `"user"` (PostgreSQL), `` `user` `` (MySQL), `[user]` (SQLServer)

**And** `limit_offset_sql(limit, offset)` gibt plattform-korrektes SQL
**And** `current_timestamp_sql()` gibt NOW/CURRENT_TIMESTAMP
**And** Type-Declarations sind plattform-spezifisch

**Technical Notes:**
- `Platform` trait in `rustine-platform/src/lib.rs`
- Default implementations wo möglich
- `Arc<dyn Platform>` für Sharing

**FRs:** FR29, FR30, FR31

---

### Story 4.2: PostgreSQL Platform

Als **Library-Nutzer**,
möchte ich **PostgreSQL-spezifisches SQL generieren**,
damit ich **alle PostgreSQL-Features nutzen kann**.

**Acceptance Criteria:**

**Given** ich nutze PostgreSQL
**When** ich einen Boolean-Type deklariere
**Then** wird `BOOLEAN` verwendet (nicht TINYINT)

**And** Identifier werden mit `"` gequotet
**And** JSONB wird für JSON-Columns unterstützt
**And** SERIAL/BIGSERIAL für Auto-Increment
**And** PostgreSQL 12+ wird unterstützt

**Technical Notes:**
- `PostgresPlatform` struct implementing `Platform`
- Version-Detection für Features (z.B. GENERATED ALWAYS)
- Introspection-SQL für pg_catalog

**FRs:** FR28, FR32

---

### Story 4.3: MySQL Platform

Als **Library-Nutzer**,
möchte ich **MySQL-spezifisches SQL generieren**,
damit ich **MySQL/MariaDB nutzen kann**.

**Acceptance Criteria:**

**Given** ich nutze MySQL
**When** ich einen Boolean-Type deklariere
**Then** wird `TINYINT(1)` verwendet

**And** Identifier werden mit `` ` `` gequotet
**And** AUTO_INCREMENT für Auto-Increment
**And** `supports_release_savepoints() = false`
**And** MySQL 8.0+ und MariaDB 10.5+ werden unterstützt

**Technical Notes:**
- `MySqlPlatform` struct implementing `Platform`
- MariaDB-Detection über Server-Version-String
- Introspection-SQL für information_schema

**FRs:** FR28, FR32

---

### Story 4.4: SQLite Platform

Als **Library-Nutzer**,
möchte ich **SQLite-spezifisches SQL generieren**,
damit ich **SQLite für Development/Testing nutzen kann**.

**Acceptance Criteria:**

**Given** ich nutze SQLite
**When** ich einen Auto-Increment-Integer deklariere
**Then** wird `INTEGER PRIMARY KEY` verwendet

**And** Identifier werden mit `"` gequotet (wie PostgreSQL)
**And** Keine echten Boolean-Types (INTEGER 0/1)
**And** SQLite 3.35+ wird unterstützt (für RETURNING)

**Technical Notes:**
- `SqlitePlatform` struct implementing `Platform`
- Introspection via `sqlite_master` und `PRAGMA`
- Eingeschränkte ALTER TABLE Unterstützung dokumentieren

**FRs:** FR28, FR32

---

## Epic 5: Query Builder

**Goal:** Ermögliche programmatische Query-Konstruktion.

**Crate:** `rustine-query`

**Prerequisites:** Epic 4 (Platform Abstraction)

---

### Story 5.1: SELECT Query Builder

Als **Library-Nutzer**,
möchte ich **SELECT-Queries programmatisch bauen**,
damit ich **dynamische Queries typsicher erstellen kann**.

**Acceptance Criteria:**

**Given** ich möchte eine SELECT-Query bauen
**When** ich `qb.select(&["id", "name"]).from("users", Some("u")).fetch_all().await` aufrufe
**Then** wird `SELECT id, name FROM users u` generiert und ausgeführt

**And** `.distinct()` fügt DISTINCT hinzu
**And** `.where_clause(expr)` fügt WHERE hinzu
**And** `.and_where(expr)` verkettet mit AND
**And** `.or_where(expr)` verkettet mit OR
**And** `.order_by("name", Order::Asc)` fügt ORDER BY hinzu
**And** `.group_by(&["status"])` fügt GROUP BY hinzu
**And** `.having(expr)` fügt HAVING hinzu
**And** `.limit(10).offset(5)` fügt LIMIT/OFFSET hinzu

**Technical Notes:**
- `QueryBuilder<'conn>` mit owned builder pattern
- State enum für Query-Type
- SQL-Generation delegiert an Platform

**FRs:** FR18, FR23, FR24, FR25, FR27

---

### Story 5.2: INSERT Query Builder

Als **Library-Nutzer**,
möchte ich **INSERT-Queries programmatisch bauen**,
damit ich **Daten typsicher einfügen kann**.

**Acceptance Criteria:**

**Given** ich möchte Daten einfügen
**When** ich `qb.insert("users").values(&[("name", ":name")]).set_parameter("name", "John").execute().await` aufrufe
**Then** wird `INSERT INTO users (name) VALUES ($1)` generiert

**And** Multiple rows können mit `.values_batch()` eingefügt werden
**And** `execute()` gibt affected rows zurück

**Technical Notes:**
- INSERT-State im QueryBuilder
- Values als `Vec<(String, String)>` (column, placeholder)
- Batch-Insert für Performance

**FRs:** FR19, FR27

---

### Story 5.3: UPDATE Query Builder

Als **Library-Nutzer**,
möchte ich **UPDATE-Queries programmatisch bauen**,
damit ich **Daten typsicher aktualisieren kann**.

**Acceptance Criteria:**

**Given** ich möchte Daten aktualisieren
**When** ich `qb.update("users").set("name", ":name").where_clause(expr::eq("id", ":id")).execute().await` aufrufe
**Then** wird `UPDATE users SET name = $1 WHERE id = $2` generiert

**And** Multiple `.set()` Aufrufe sind möglich
**And** WHERE ist pflicht (Safety)
**And** `execute()` gibt affected rows zurück

**Technical Notes:**
- UPDATE-State im QueryBuilder
- Set als `Vec<(String, String)>`
- Warning/Error wenn kein WHERE (optional)

**FRs:** FR20, FR27

---

### Story 5.4: DELETE Query Builder

Als **Library-Nutzer**,
möchte ich **DELETE-Queries programmatisch bauen**,
damit ich **Daten typsicher löschen kann**.

**Acceptance Criteria:**

**Given** ich möchte Daten löschen
**When** ich `qb.delete("users").where_clause(expr::eq("id", ":id")).execute().await` aufrufe
**Then** wird `DELETE FROM users WHERE id = $1` generiert

**And** WHERE ist pflicht (Safety, kein DELETE ohne WHERE)
**And** `execute()` gibt affected rows zurück

**Technical Notes:**
- DELETE-State im QueryBuilder
- Hard-Error wenn kein WHERE

**FRs:** FR21, FR27

---

### Story 5.5: JOIN Support

Als **Library-Nutzer**,
möchte ich **JOINs im QueryBuilder verwenden**,
damit ich **Daten aus mehreren Tabellen verknüpfen kann**.

**Acceptance Criteria:**

**Given** ich möchte Tabellen joinen
**When** ich `.inner_join("posts", "p", "p.user_id = u.id")` aufrufe
**Then** wird `INNER JOIN posts p ON p.user_id = u.id` generiert

**And** `.left_join()` für LEFT JOIN
**And** `.right_join()` für RIGHT JOIN
**And** Multiple JOINs sind möglich

**Technical Notes:**
- `JoinClause` struct mit type, table, alias, condition
- Joins werden in FROM-Order generiert

**FRs:** FR22

---

### Story 5.6: Expression Builder

Als **Library-Nutzer**,
möchte ich **WHERE-Conditions typsicher bauen**,
damit ich **komplexe Bedingungen ohne String-Konkatenation erstellen kann**.

**Acceptance Criteria:**

**Given** ich brauche eine komplexe WHERE-Bedingung
**When** ich `expr::and(vec![expr::eq("status", ":status"), expr::gt("age", ":age")])` aufrufe
**Then** wird `(status = $1 AND age > $2)` generiert

**And** `expr::eq()`, `expr::neq()`, `expr::lt()`, `expr::lte()`, `expr::gt()`, `expr::gte()`
**And** `expr::is_null()`, `expr::is_not_null()`
**And** `expr::like()`, `expr::not_like()`
**And** `expr::in_list()`, `expr::not_in_list()`
**And** `expr::and()`, `expr::or()` für Komposition
**And** `expr::between()` für Ranges

**Technical Notes:**
- `Expression` enum für verschiedene Typen
- `expr` module mit Builder-Functions
- Recursive composition für AND/OR

**FRs:** FR26

---

## Epic 6: Schema Introspection

**Goal:** Ermögliche das Auslesen von Datenbank-Schemas.

**Crate:** `rustine-schema`

**Prerequisites:** Epic 2 (Database Connectivity), Epic 4 (Platform)

---

### Story 6.1: List Tables

Als **Library-Nutzer**,
möchte ich **alle Tabellen einer Datenbank auflisten**,
damit ich **das Schema erkunden kann**.

**Acceptance Criteria:**

**Given** ich habe eine verbundene Connection
**When** ich `schema_manager.list_table_names().await` aufrufe
**Then** erhalte ich `Vec<String>` mit Tabellennamen

**And** `table_exists("users").await` prüft ob Tabelle existiert
**And** System-Tabellen werden gefiltert (optional)

**Technical Notes:**
- `SchemaManager` trait
- Platform-spezifische Introspection-Queries
- PostgreSQL: `information_schema.tables`
- MySQL: `information_schema.tables`
- SQLite: `sqlite_master`

**FRs:** FR33, FR38

---

### Story 6.2: Introspect Columns

Als **Library-Nutzer**,
möchte ich **Spalten einer Tabelle introspizieren**,
damit ich **die Tabellenstruktur verstehen kann**.

**Acceptance Criteria:**

**Given** ich habe eine Tabelle
**When** ich `schema_manager.list_table_columns("users").await` aufrufe
**Then** erhalte ich `Vec<Column>` mit allen Spalten

**And** `Column` enthält: name, data_type, nullable, default, auto_increment
**And** `column.column_type()` gibt abstrahierten ColumnType

**Technical Notes:**
- `Column` struct
- `ColumnType` enum (Integer, String, Text, Boolean, DateTime, etc.)
- Platform-spezifisches Type-Mapping

**FRs:** FR34

---

### Story 6.3: Introspect Indexes & Foreign Keys

Als **Library-Nutzer**,
möchte ich **Indexes und Foreign Keys einer Tabelle auslesen**,
damit ich **Constraints und Performance-Optimierungen verstehen kann**.

**Acceptance Criteria:**

**Given** ich habe eine Tabelle
**When** ich `schema_manager.list_table_indexes("users").await` aufrufe
**Then** erhalte ich `Vec<Index>` mit allen Indexes

**When** ich `schema_manager.list_table_foreign_keys("users").await` aufrufe
**Then** erhalte ich `Vec<ForeignKey>` mit allen FKs

**And** `Index` enthält: name, columns, unique, primary
**And** `ForeignKey` enthält: name, local_columns, foreign_table, foreign_columns, on_delete, on_update

**Technical Notes:**
- `Index` und `ForeignKey` structs
- `ReferentialAction` enum (Cascade, SetNull, Restrict, NoAction)

**FRs:** FR35, FR36

---

### Story 6.4: Full Schema Introspection

Als **Library-Nutzer**,
möchte ich **das komplette Schema als Objekt-Graph laden**,
damit ich **das gesamte Datenbankschema analysieren kann**.

**Acceptance Criteria:**

**Given** ich habe eine Datenbank
**When** ich `schema_manager.introspect_schema().await` aufrufe
**Then** erhalte ich ein `Schema` Objekt mit allen Tabellen

**And** `Schema` enthält: `tables: HashMap<String, Table>`
**And** `Table` enthält: columns, indexes, foreign_keys, primary_key
**And** `introspect_table("users")` lädt nur eine Tabelle

**Technical Notes:**
- `Schema` struct als Container
- `Table` struct mit allen Details
- Lazy-loading Option für große Schemas

**FRs:** FR37

---

## FR Coverage Matrix

| FR | Epic | Story | Status |
|----|------|-------|--------|
| FR1 | 2 | 2.1 | ⬜ |
| FR2 | 2 | 2.1 | ⬜ |
| FR3 | 2 | 2.2 | ⬜ |
| FR4 | 2 | 2.2 | ⬜ |
| FR5 | 2 | 2.2 | ⬜ |
| FR6 | 2 | 2.3 | ⬜ |
| FR7 | 2 | 2.4 | ⬜ |
| FR8 | 2 | 2.3 | ⬜ |
| FR9 | 2 | 2.5 | ⬜ |
| FR10 | 2 | 2.5 | ⬜ |
| FR11 | 2 | 2.5 | ⬜ |
| FR12 | 2 | 2.3 | ⬜ |
| FR13 | 3 | 3.1 | ⬜ |
| FR14 | 3 | 3.2 | ⬜ |
| FR15 | 3 | 3.3 | ⬜ |
| FR16 | 3 | 3.4 | ⬜ |
| FR17 | 3 | 3.1 | ⬜ |
| FR18 | 5 | 5.1 | ⬜ |
| FR19 | 5 | 5.2 | ⬜ |
| FR20 | 5 | 5.3 | ⬜ |
| FR21 | 5 | 5.4 | ⬜ |
| FR22 | 5 | 5.5 | ⬜ |
| FR23 | 5 | 5.1 | ⬜ |
| FR24 | 5 | 5.1 | ⬜ |
| FR25 | 5 | 5.1 | ⬜ |
| FR26 | 5 | 5.6 | ⬜ |
| FR27 | 5 | 5.1-5.4 | ⬜ |
| FR28 | 4 | 4.2-4.4 | ⬜ |
| FR29 | 4 | 4.1 | ⬜ |
| FR30 | 4 | 4.1 | ⬜ |
| FR31 | 4 | 4.1 | ⬜ |
| FR32 | 4 | 4.2-4.4 | ⬜ |
| FR33 | 6 | 6.1 | ⬜ |
| FR34 | 6 | 6.2 | ⬜ |
| FR35 | 6 | 6.3 | ⬜ |
| FR36 | 6 | 6.3 | ⬜ |
| FR37 | 6 | 6.4 | ⬜ |
| FR38 | 6 | 6.1 | ⬜ |
| FR39 | 1 | 1.3 | ⬜ |
| FR40 | 1 | 1.4 | ⬜ |
| FR41 | 1 | 1.3 | ⬜ |
| FR42 | 1 | 1.5 | ⬜ |
| FR43 | 1 | 1.5 | ⬜ |
| FR44 | 1 | 1.5 | ⬜ |
| FR45 | 1 | 1.5 | ⬜ |
| FR46 | 1 | 1.1 | ⬜ |
| FR47 | 1 | 1.1 | ⬜ |
| FR48 | 1 | 1.2 | ⬜ |
| FR49 | 1 | 1.2 | ⬜ |

---

## Summary

### Implementation Order

1. **Epic 1: Core Foundation** (keine Dependencies)
   - Stories 1.1-1.6 in Reihenfolge
   - Ergebnis: `rustine-core` Crate funktionsfähig

2. **Epic 4: Platform Abstraction** (benötigt Epic 1)
   - Story 4.1 zuerst (Trait)
   - Stories 4.2-4.4 parallel möglich
   - Ergebnis: `rustine-platform` Crate funktionsfähig

3. **Epic 2: Database Connectivity** (benötigt Epic 1, 4)
   - Stories 2.1-2.5 in Reihenfolge
   - Ergebnis: Basis-Queries funktionieren

4. **Epic 3: Transaction Management** (benötigt Epic 2)
   - Stories 3.1-3.4 in Reihenfolge
   - Ergebnis: Transaktionen funktionieren

5. **Epic 5: Query Builder** (benötigt Epic 4)
   - Story 5.6 (Expressions) zuerst
   - Stories 5.1-5.5 danach
   - Ergebnis: `rustine-query` Crate funktionsfähig

6. **Epic 6: Schema Introspection** (benötigt Epic 2, 4)
   - Stories 6.1-6.4 in Reihenfolge
   - Ergebnis: `rustine-schema` Crate funktionsfähig

### Parallel Work Streams

```
Epic 1 ──┬──► Epic 4 ──┬──► Epic 5
         │             │
         └──► Epic 2 ──┴──► Epic 3
                       │
                       └──► Epic 6
```

---

*Dieses Epic Breakdown basiert auf der BMAD Method v6 und definiert die implementierbaren Einheiten für Rustine.*
