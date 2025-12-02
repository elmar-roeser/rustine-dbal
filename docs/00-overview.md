# Doctrine DBAL Architektur-Übersicht

Dieses Dokument beschreibt die Architektur von Doctrine DBAL als Referenz für die Rust-Portierung.

## Schichten-Architektur

```
┌─────────────────────────────────────────────────────────────────┐
│                        Public API                               │
│  Connection, QueryBuilder, SchemaManager                        │
├─────────────────────────────────────────────────────────────────┤
│                     Abstraction Layer                           │
│  Platform (SQL-Dialekte), Types (Konvertierung)                 │
├─────────────────────────────────────────────────────────────────┤
│                      Driver Layer                               │
│  Driver, Driver\Connection, Driver\Statement, Driver\Result     │
├─────────────────────────────────────────────────────────────────┤
│                   Native PHP Extensions                         │
│  PDO, mysqli, pgsql, oci8, sqlsrv, ibm_db2, sqlite3            │
└─────────────────────────────────────────────────────────────────┘
```

## Komponenten-Übersicht

| Komponente | PHP-Datei | Verantwortlichkeit |
|------------|-----------|-------------------|
| Connection | `src/Connection.php` | Hauptzugriffspunkt, Transactions, Query-Ausführung |
| Driver | `src/Driver.php` | Interface für DB-Treiber |
| Platform | `src/Platforms/AbstractPlatform.php` | SQL-Dialekt-Generierung |
| SchemaManager | `src/Schema/AbstractSchemaManager.php` | DB-Introspection, DDL |
| QueryBuilder | `src/Query/QueryBuilder.php` | Programmatische Query-Konstruktion |
| Type | `src/Types/Type.php` | Wert-Konvertierung PHP ↔ DB |
| Result | `src/Result.php` | Query-Ergebnis-Iteration |

## Datenfluss

### Query-Ausführung

```
User Code
    │
    ▼
Connection::executeQuery(sql, params, types)
    │
    ├─► expandArrayParameters()     // Array-Parameter IN (?) expandieren
    │
    ├─► Driver\Connection::prepare()
    │
    ├─► bindParameters()            // Type-Konvertierung via Type::convertToDatabaseValue()
    │
    ├─► Driver\Statement::execute()
    │
    └─► Result (wrapping Driver\Result)
```

### Schema-Introspection

```
User Code
    │
    ▼
Connection::createSchemaManager()
    │
    ▼
SchemaManager::listTables()
    │
    ├─► Platform::getListTablesSQL()    // Platform-spezifisches SQL
    │
    ├─► Connection::fetchAllAssociative()
    │
    └─► _getPortableTableDefinition()   // Normalisierung
```

## Verzeichnisstruktur

```
dbal/src/
├── Connection.php              # Haupt-Connection-Klasse
├── Driver.php                  # Driver-Interface
├── DriverManager.php           # Connection-Factory
├── Result.php                  # Query-Result-Wrapper
├── Statement.php               # Prepared Statement Wrapper
├── Configuration.php           # Connection-Konfiguration
│
├── Driver/                     # Treiber-Implementierungen
│   ├── Connection.php          # Driver-Connection-Interface
│   ├── Statement.php           # Driver-Statement-Interface
│   ├── Result.php              # Driver-Result-Interface
│   ├── PDO/                    # PDO-basierte Treiber
│   ├── Mysqli/                 # MySQLi-Treiber
│   ├── PgSQL/                  # PostgreSQL native
│   ├── SQLite3/                # SQLite3 native
│   ├── OCI8/                   # Oracle OCI8
│   ├── SQLSrv/                 # SQL Server
│   └── IBMDB2/                 # IBM DB2
│
├── Platforms/                  # SQL-Dialekt-Implementierungen
│   ├── AbstractPlatform.php    # Basis-Klasse
│   ├── MySQLPlatform.php
│   ├── PostgreSQLPlatform.php
│   ├── SQLitePlatform.php
│   ├── OraclePlatform.php
│   ├── SQLServerPlatform.php
│   └── DB2Platform.php
│
├── Schema/                     # Schema-Management
│   ├── AbstractSchemaManager.php
│   ├── Table.php, Column.php, Index.php, ...
│   ├── Comparator.php          # Schema-Diff
│   └── SchemaDiff.php
│
├── Query/                      # Query Builder
│   ├── QueryBuilder.php
│   └── Expression/
│
└── Types/                      # Type-System
    ├── Type.php
    ├── StringType.php, IntegerType.php, ...
    └── TypeRegistry.php
```

## Dokumentation dieser Analyse

- [01-connection.md](01-connection.md) - Connection-Komponente
- [02-driver.md](02-driver.md) - Driver-Layer
- [03-platform.md](03-platform.md) - Platform/SQL-Dialekte
- [04-query-builder.md](04-query-builder.md) - QueryBuilder
- [05-schema.md](05-schema.md) - Schema-Management
- [06-types.md](06-types.md) - Type-System
- [07-rust-mapping.md](07-rust-mapping.md) - Rust-Portierungs-Strategie
