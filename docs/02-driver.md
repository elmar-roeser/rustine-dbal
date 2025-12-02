# Driver Layer

Der Driver-Layer abstrahiert die verschiedenen PHP-Datenbank-Erweiterungen hinter einheitlichen Interfaces.

## Interface-Hierarchie

```
Driver (Factory)
    │
    ├── connect(params) → Driver\Connection
    ├── getDatabasePlatform() → AbstractPlatform
    └── getExceptionConverter() → ExceptionConverter

Driver\Connection
    │
    ├── prepare(sql) → Driver\Statement
    ├── query(sql) → Driver\Result
    ├── exec(sql) → int
    ├── quote(string) → string
    ├── lastInsertId() → int|string
    ├── beginTransaction()
    ├── commit()
    ├── rollBack()
    ├── getServerVersion() → string
    └── getNativeConnection()

Driver\Statement
    │
    ├── bindValue(param, value, type)
    └── execute() → Driver\Result

Driver\Result
    │
    ├── fetchNumeric() → array|false
    ├── fetchAssociative() → array|false
    ├── fetchOne() → mixed|false
    ├── fetchAllNumeric() → array
    ├── fetchAllAssociative() → array
    ├── fetchFirstColumn() → array
    ├── rowCount() → int|string
    ├── columnCount() → int
    ├── getColumnName(index) → string
    └── free()
```

## Driver Interface

```php
interface Driver
{
    /**
     * Stellt Verbindung her
     * @throws Exception bei Verbindungsfehler
     */
    public function connect(array $params): DriverConnection;

    /**
     * Gibt Platform basierend auf Server-Version zurück
     */
    public function getDatabasePlatform(ServerVersionProvider $versionProvider): AbstractPlatform;

    /**
     * Konvertiert Driver-Exceptions zu DBAL-Exceptions
     */
    public function getExceptionConverter(): ExceptionConverter;
}
```

## Driver\Connection Interface

```php
interface Connection extends ServerVersionProvider
{
    public function prepare(string $sql): Statement;
    public function query(string $sql): Result;
    public function quote(string $value): string;
    public function exec(string $sql): int|string;
    public function lastInsertId(): int|string;

    public function beginTransaction(): void;
    public function commit(): void;
    public function rollBack(): void;

    public function getServerVersion(): string;
    public function getNativeConnection();
}
```

## Driver\Statement Interface

```php
interface Statement
{
    /**
     * @param int|string $param  Position (1-basiert) oder Name (:name)
     * @param mixed $value       Der zu bindende Wert
     * @param ParameterType $type  Typ-Hint für Binding
     */
    public function bindValue(int|string $param, mixed $value, ParameterType $type): void;

    public function execute(): Result;
}
```

## Driver\Result Interface

```php
interface Result
{
    /** @return list<mixed>|false */
    public function fetchNumeric(): array|false;

    /** @return array<string,mixed>|false */
    public function fetchAssociative(): array|false;

    public function fetchOne(): mixed;

    /** @return list<list<mixed>> */
    public function fetchAllNumeric(): array;

    /** @return list<array<string,mixed>> */
    public function fetchAllAssociative(): array;

    /** @return list<mixed> */
    public function fetchFirstColumn(): array;

    /** @return int|numeric-string */
    public function rowCount(): int|string;

    public function columnCount(): int;

    public function getColumnName(int $index): string;

    public function free(): void;
}
```

## ParameterType Enum

```php
enum ParameterType: int
{
    case NULL = 0;
    case INTEGER = 1;
    case STRING = 2;
    case LARGE_OBJECT = 3;  // BLOB
    case BOOLEAN = 5;
    case BINARY = 16;
    case ASCII = 17;
}
```

## Verfügbare Driver

| Driver | PHP Extension | Klasse |
|--------|--------------|--------|
| pdo_mysql | PDO + pdo_mysql | `PDO\MySQL\Driver` |
| pdo_pgsql | PDO + pdo_pgsql | `PDO\PgSQL\Driver` |
| pdo_sqlite | PDO + pdo_sqlite | `PDO\SQLite\Driver` |
| pdo_sqlsrv | PDO + pdo_sqlsrv | `PDO\SQLSrv\Driver` |
| pdo_oci | PDO + pdo_oci | `PDO\OCI\Driver` |
| mysqli | mysqli | `Mysqli\Driver` |
| pgsql | pgsql | `PgSQL\Driver` |
| sqlite3 | sqlite3 | `SQLite3\Driver` |
| sqlsrv | sqlsrv | `SQLSrv\Driver` |
| oci8 | oci8 | `OCI8\Driver` |
| ibm_db2 | ibm_db2 | `IBMDB2\Driver` |

## Abstract Driver Klassen

Jede Datenbank hat einen Abstract-Driver der gemeinsame Logik enthält:

```php
abstract class AbstractMySQLDriver implements Driver
{
    public function getDatabasePlatform(ServerVersionProvider $versionProvider): AbstractPlatform
    {
        $version = $versionProvider->getServerVersion();
        $mariaDB = stripos($version, 'mariadb') !== false;

        if ($mariaDB) {
            // MariaDB-Version parsen und passende Platform zurückgeben
            return new MariaDBPlatform();  // oder spezifische Version
        }

        // MySQL-Version parsen
        return new MySQLPlatform();  // oder MySQL80Platform, etc.
    }

    public function getExceptionConverter(): ExceptionConverter
    {
        return new MySQL\ExceptionConverter();
    }
}
```

## Exception Handling

### ExceptionConverter Interface

```php
interface ExceptionConverter
{
    public function convert(Exception $exception, ?Query $query): DriverException;
}
```

### Exception-Hierarchie

```
DriverException
├── ConnectionException
│   └── ConnectionLost
├── ConstraintViolationException
│   ├── ForeignKeyConstraintViolationException
│   ├── NotNullConstraintViolationException
│   └── UniqueConstraintViolationException
├── DatabaseObjectExistsException
├── DatabaseObjectNotFoundException
├── DeadlockException
├── InvalidFieldNameException
├── LockWaitTimeoutException
├── NonUniqueFieldNameException
├── ReadOnlyException
├── ServerException
├── SyntaxErrorException
└── TableExistsException / TableNotFoundException
```

## Middleware-System

Driver können durch Middleware gewrappt werden:

```php
interface Middleware
{
    public function wrap(Driver $driver): Driver;
}
```

Beispiel: Logging-Middleware

```php
class LoggingMiddleware implements Middleware
{
    public function wrap(Driver $driver): Driver
    {
        return new LoggingDriver($driver, $this->logger);
    }
}
```

## Rust-Portierung

### Trait-Design

```rust
pub trait Driver: Send + Sync {
    type Connection: DriverConnection;
    type Error: std::error::Error;

    async fn connect(&self, params: &ConnectionParams) -> Result<Self::Connection, Self::Error>;

    fn platform(&self, version: &str) -> Box<dyn Platform>;

    fn exception_converter(&self) -> Box<dyn ExceptionConverter>;
}

pub trait DriverConnection: Send {
    type Statement<'a>: DriverStatement<'a> where Self: 'a;
    type Result: DriverResult;

    async fn prepare(&self, sql: &str) -> Result<Self::Statement<'_>, Error>;
    async fn query(&self, sql: &str) -> Result<Self::Result, Error>;
    async fn exec(&self, sql: &str) -> Result<u64, Error>;

    fn quote(&self, value: &str) -> String;
    async fn last_insert_id(&self) -> Result<i64, Error>;

    async fn begin_transaction(&mut self) -> Result<(), Error>;
    async fn commit(&mut self) -> Result<(), Error>;
    async fn rollback(&mut self) -> Result<(), Error>;

    async fn server_version(&self) -> Result<String, Error>;
}

pub trait DriverStatement<'conn>: Send {
    type Result: DriverResult;

    fn bind_value<V: ToSql>(&mut self, index: usize, value: V, param_type: ParameterType);

    async fn execute(&mut self) -> Result<Self::Result, Error>;
}

pub trait DriverResult: Send {
    fn fetch_one(&mut self) -> Option<Row>;
    fn fetch_all(self) -> Vec<Row>;

    fn row_count(&self) -> u64;
    fn column_count(&self) -> usize;
    fn column_name(&self, index: usize) -> Option<&str>;
}
```

### Implementierung mit sqlx

```rust
pub struct SqlxPostgresDriver;

impl Driver for SqlxPostgresDriver {
    type Connection = SqlxPostgresConnection;
    type Error = sqlx::Error;

    async fn connect(&self, params: &ConnectionParams) -> Result<Self::Connection, Self::Error> {
        let pool = PgPoolOptions::new()
            .connect(&params.to_connection_string())
            .await?;
        Ok(SqlxPostgresConnection::new(pool))
    }

    fn platform(&self, version: &str) -> Box<dyn Platform> {
        // Version parsen und passende Platform zurückgeben
        Box::new(PostgreSQLPlatform::new())
    }
}
```

### ParameterType Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    Null,
    Integer,
    String,
    LargeObject,
    Boolean,
    Binary,
    Ascii,
}
```
