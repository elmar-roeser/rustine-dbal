# Platform Layer

Die Platform-Klassen generieren SQL für verschiedene Datenbank-Dialekte. Sie sind "passive" - sie führen kein SQL aus, sondern generieren es nur.

## Vererbungshierarchie

```
AbstractPlatform
├── MySQLPlatform
│   ├── MySQL80Platform
│   └── MySQL84Platform
├── MariaDBPlatform
│   ├── MariaDB1010Platform
│   ├── MariaDB1052Platform
│   ├── MariaDB1060Platform
│   └── MariaDB110700Platform
├── PostgreSQLPlatform
│   └── PostgreSQL120Platform
├── SQLitePlatform
├── OraclePlatform
├── SQLServerPlatform
└── DB2Platform
```

## Hauptverantwortlichkeiten

### 1. SQL-Type-Deklarationen

```php
abstract public function getBooleanTypeDeclarationSQL(array $column): string;
abstract public function getIntegerTypeDeclarationSQL(array $column): string;
abstract public function getBigIntTypeDeclarationSQL(array $column): string;
abstract public function getSmallIntTypeDeclarationSQL(array $column): string;

public function getStringTypeDeclarationSQL(array $column): string;
public function getBinaryTypeDeclarationSQL(array $column): string;
public function getGuidTypeDeclarationSQL(array $column): string;
public function getJsonTypeDeclarationSQL(array $column): string;
public function getClobTypeDeclarationSQL(array $column): string;
public function getBlobTypeDeclarationSQL(array $column): string;
public function getDateTypeDeclarationSQL(array $column): string;
public function getTimeTypeDeclarationSQL(array $column): string;
public function getDateTimeTypeDeclarationSQL(array $column): string;
```

### 2. Identifier Quoting

```php
// Verschiedene Quoting-Styles
// MySQL:      `identifier`
// PostgreSQL: "identifier"
// SQL Server: [identifier]

public function quoteSingleIdentifier(string $str): string
{
    $c = $this->getIdentifierQuoteCharacter();
    return $c . str_replace($c, $c . $c, $str) . $c;
}

// Beispiel PostgreSQL:
protected function getIdentifierQuoteCharacter(): string
{
    return '"';
}
```

### 3. DDL-Generierung

```php
// CREATE TABLE
public function getCreateTableSQL(Table $table, int $createFlags = self::CREATE_INDEXES): array;

// ALTER TABLE
public function getAlterTableSQL(TableDiff $diff): array;

// DROP TABLE
public function getDropTableSQL(string $table): string;

// Indexes
public function getCreateIndexSQL(Index $index, string $table): string;
public function getDropIndexSQL(string $index, string $table): string;

// Foreign Keys
public function getCreateForeignKeySQL(ForeignKeyConstraint $fk, string $table): string;
public function getDropForeignKeySQL(string $fk, string $table): string;

// Sequences
public function getCreateSequenceSQL(Sequence $sequence): string;
public function getDropSequenceSQL(string $sequence): string;

// Views
public function getCreateViewSQL(string $name, string $sql): string;
public function getDropViewSQL(string $name): string;
```

### 4. DML-Unterschiede

```php
// LIMIT/OFFSET
public function modifyLimitQuery(string $query, ?int $limit, int $offset = 0): string;

// MySQL:      ... LIMIT 10 OFFSET 5
// PostgreSQL: ... LIMIT 10 OFFSET 5
// SQL Server: ... OFFSET 5 ROWS FETCH NEXT 10 ROWS ONLY
// Oracle:     komplexe Subquery mit ROWNUM

// CONCAT
public function getConcatExpression(string ...$strings): string;
// MySQL:      CONCAT(a, b, c)
// SQL Server: a + b + c

// SUBSTRING
public function getSubstringExpression(string $string, string $start, ?string $length = null): string;
// MySQL:      SUBSTRING(str, start, length)
// SQL Server: SUBSTRING(str, start, length)

// LENGTH
public function getLengthExpression(string $string): string;
// MySQL:      CHAR_LENGTH(str)
// PostgreSQL: LENGTH(str)
// SQL Server: LEN(str)

// TRIM
public function getTrimExpression(string $str, int $mode = TrimMode::UNSPECIFIED, ?string $char = null): string;

// LOCATE
public function getLocateExpression(string $needle, string $haystack, ?string $startPos = null): string;
// MySQL:      LOCATE(needle, haystack, start)
// SQL Server: CHARINDEX(needle, haystack, start)

// NOW/CURRENT_TIMESTAMP
public function getCurrentTimestampSQL(): string;
public function getCurrentDateSQL(): string;
public function getCurrentTimeSQL(): string;
```

### 5. Schema Introspection SQL

```php
// Diese Methoden generieren das SQL für Schema-Introspection
public function getListTablesSQL(): string;
public function getListTableColumnsSQL(string $table, string $database): string;
public function getListTableIndexesSQL(string $table, string $database): string;
public function getListTableForeignKeysSQL(string $table, string $database): string;
public function getListSequencesSQL(string $database): string;
public function getListViewsSQL(string $database): string;
```

### 6. Transaction Isolation

```php
public function getSetTransactionIsolationSQL(TransactionIsolationLevel $level): string;

// MySQL:     SET TRANSACTION ISOLATION LEVEL READ COMMITTED
// PostgreSQL: SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL READ COMMITTED
// SQL Server: SET TRANSACTION ISOLATION LEVEL READ COMMITTED
```

### 7. Savepoints

```php
public function supportsSavepoints(): bool;
public function supportsReleaseSavepoints(): bool;  // MySQL: false!

public function createSavePoint(string $savepoint): string;      // SAVEPOINT $name
public function releaseSavePoint(string $savepoint): string;     // RELEASE SAVEPOINT $name
public function rollbackSavePoint(string $savepoint): string;    // ROLLBACK TO SAVEPOINT $name
```

## Platform-spezifische Unterschiede

### AUTO_INCREMENT / IDENTITY / SERIAL

| Platform | Syntax |
|----------|--------|
| MySQL | `INT AUTO_INCREMENT` |
| PostgreSQL | `SERIAL` / `INT GENERATED BY DEFAULT AS IDENTITY` |
| SQLite | `INTEGER PRIMARY KEY AUTOINCREMENT` |
| SQL Server | `INT IDENTITY(1,1)` |
| Oracle | Sequences + Triggers |

### Boolean

| Platform | Storage | True | False |
|----------|---------|------|-------|
| MySQL | TINYINT(1) | 1 | 0 |
| PostgreSQL | BOOLEAN | TRUE | FALSE |
| SQLite | INTEGER | 1 | 0 |
| SQL Server | BIT | 1 | 0 |

### JSON

| Platform | Type | Features |
|----------|------|----------|
| MySQL 5.7+ | JSON | Native JSON |
| PostgreSQL | JSON/JSONB | JSONB mit Index-Support |
| SQLite | TEXT | JSON-Funktionen seit 3.38 |
| SQL Server | NVARCHAR(MAX) | JSON-Funktionen |

### Date/Time

```php
// Format für Literals
public function getDateTimeFormatString(): string;
// MySQL:      'Y-m-d H:i:s'
// PostgreSQL: 'Y-m-d H:i:s.u'
// SQL Server: 'Y-m-d H:i:s.v'

public function getDateFormatString(): string;   // 'Y-m-d'
public function getTimeFormatString(): string;   // 'H:i:s'
```

## Type-Mapping (DB → Doctrine)

```php
protected function initializeDoctrineTypeMappings(): void
{
    // MySQL Beispiel
    $this->doctrineTypeMapping = [
        'tinyint'    => Types::BOOLEAN,  // Achtung: nur für tinyint(1)
        'smallint'   => Types::SMALLINT,
        'mediumint'  => Types::INTEGER,
        'int'        => Types::INTEGER,
        'bigint'     => Types::BIGINT,
        'float'      => Types::FLOAT,
        'double'     => Types::FLOAT,
        'decimal'    => Types::DECIMAL,
        'varchar'    => Types::STRING,
        'char'       => Types::STRING,
        'text'       => Types::TEXT,
        'blob'       => Types::BLOB,
        'date'       => Types::DATE_MUTABLE,
        'datetime'   => Types::DATETIME_MUTABLE,
        'timestamp'  => Types::DATETIME_MUTABLE,
        'time'       => Types::TIME_MUTABLE,
        'json'       => Types::JSON,
        // ...
    ];
}
```

## Rust-Portierung

### Trait-Design

```rust
pub trait Platform: Send + Sync {
    // === Type Declarations ===
    fn boolean_type_sql(&self, column: &ColumnDefinition) -> String;
    fn integer_type_sql(&self, column: &ColumnDefinition) -> String;
    fn bigint_type_sql(&self, column: &ColumnDefinition) -> String;
    fn string_type_sql(&self, column: &ColumnDefinition) -> String;
    fn text_type_sql(&self, column: &ColumnDefinition) -> String;
    fn binary_type_sql(&self, column: &ColumnDefinition) -> String;
    fn json_type_sql(&self, column: &ColumnDefinition) -> String;
    fn datetime_type_sql(&self, column: &ColumnDefinition) -> String;
    fn date_type_sql(&self, column: &ColumnDefinition) -> String;
    fn time_type_sql(&self, column: &ColumnDefinition) -> String;

    // === Identifier Quoting ===
    fn quote_identifier(&self, identifier: &str) -> String;
    fn identifier_quote_char(&self) -> char;

    // === DDL ===
    fn create_table_sql(&self, table: &Table) -> Vec<String>;
    fn alter_table_sql(&self, diff: &TableDiff) -> Vec<String>;
    fn drop_table_sql(&self, table: &str) -> String;
    fn create_index_sql(&self, index: &Index, table: &str) -> String;
    fn drop_index_sql(&self, index: &str, table: &str) -> String;

    // === DML Differences ===
    fn modify_limit_query(&self, query: &str, limit: Option<u64>, offset: u64) -> String;
    fn concat_expression(&self, strings: &[&str]) -> String;
    fn substring_expression(&self, string: &str, start: &str, length: Option<&str>) -> String;
    fn length_expression(&self, string: &str) -> String;
    fn now_expression(&self) -> String;

    // === Schema Introspection SQL ===
    fn list_tables_sql(&self) -> String;
    fn list_table_columns_sql(&self, table: &str, database: &str) -> String;
    fn list_table_indexes_sql(&self, table: &str, database: &str) -> String;
    fn list_table_foreign_keys_sql(&self, table: &str, database: &str) -> String;

    // === Transactions ===
    fn supports_savepoints(&self) -> bool { true }
    fn supports_release_savepoints(&self) -> bool { true }
    fn create_savepoint_sql(&self, name: &str) -> String;
    fn release_savepoint_sql(&self, name: &str) -> String;
    fn rollback_savepoint_sql(&self, name: &str) -> String;
    fn set_transaction_isolation_sql(&self, level: TransactionIsolationLevel) -> String;

    // === Type Mapping ===
    fn doctrine_type_mapping(&self) -> &HashMap<String, &'static str>;
}
```

### Enum für Platform-Auswahl

```rust
pub enum DatabasePlatform {
    MySQL(MySQLVersion),
    MariaDB(MariaDBVersion),
    PostgreSQL(PostgreSQLVersion),
    SQLite,
    SQLServer,
    Oracle,
    DB2,
}

impl DatabasePlatform {
    pub fn as_platform(&self) -> Box<dyn Platform> {
        match self {
            Self::PostgreSQL(v) => Box::new(PostgreSQLPlatform::new(*v)),
            Self::MySQL(v) => Box::new(MySQLPlatform::new(*v)),
            // ...
        }
    }
}
```

### Beispiel: PostgreSQL Platform

```rust
pub struct PostgreSQLPlatform {
    version: PostgreSQLVersion,
}

impl Platform for PostgreSQLPlatform {
    fn identifier_quote_char(&self) -> char { '"' }

    fn quote_identifier(&self, identifier: &str) -> String {
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    fn boolean_type_sql(&self, _column: &ColumnDefinition) -> String {
        "BOOLEAN".to_string()
    }

    fn json_type_sql(&self, column: &ColumnDefinition) -> String {
        if column.options.get("jsonb").is_some() {
            "JSONB".to_string()
        } else {
            "JSON".to_string()
        }
    }

    fn modify_limit_query(&self, query: &str, limit: Option<u64>, offset: u64) -> String {
        let mut result = query.to_string();
        if let Some(l) = limit {
            result.push_str(&format!(" LIMIT {}", l));
        }
        if offset > 0 {
            result.push_str(&format!(" OFFSET {}", offset));
        }
        result
    }

    fn create_savepoint_sql(&self, name: &str) -> String {
        format!("SAVEPOINT {}", self.quote_identifier(name))
    }

    fn supports_release_savepoints(&self) -> bool { true }

    // ...
}
```
