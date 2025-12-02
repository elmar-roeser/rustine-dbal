# Connection-Komponente

Die `Connection`-Klasse ist der zentrale Einstiegspunkt für alle Datenbankoperationen.

## Zustand (State)

```php
class Connection {
    protected ?DriverConnection $_conn = null;  // Lazy-loaded
    protected Configuration $_config;
    protected Driver $driver;

    private bool $autoCommit = true;
    private int $transactionNestingLevel = 0;
    private ?TransactionIsolationLevel $transactionIsolationLevel = null;
    private array $params;                       // Connection-Parameter
    private ?AbstractPlatform $platform = null;  // Lazy-loaded
    private bool $isRollbackOnly = false;
}
```

### State-Diagramm

```
                    ┌─────────────────┐
                    │   Disconnected  │
                    │  (_conn = null) │
                    └────────┬────────┘
                             │ connect() [lazy, on first query]
                             ▼
                    ┌─────────────────┐
           ┌───────►│   Connected     │◄───────┐
           │        │ (autoCommit=T)  │        │
           │        └────────┬────────┘        │
           │                 │ beginTransaction()
           │                 ▼                 │
           │        ┌─────────────────┐        │
           │        │  In Transaction │        │
    commit/│        │ (nesting=1)     │        │rollBack()
    rollBack()      └────────┬────────┘        │
           │                 │ beginTransaction()
           │                 ▼                 │
           │        ┌─────────────────┐        │
           └────────┤ Nested (nesting>1)├──────┘
                    │ (via Savepoints) │
                    └─────────────────┘
```

## Public API

### Connection Management

| Methode | Beschreibung | Rust-Hinweis |
|---------|--------------|--------------|
| `isConnected(): bool` | Prüft ob verbunden | `fn is_connected(&self) -> bool` |
| `close(): void` | Trennt Verbindung, reset nesting | `fn close(&mut self)` |
| `getDatabase(): ?string` | Aktueller DB-Name | `fn database(&self) -> Option<&str>` |
| `getNativeConnection()` | Zugriff auf native Connection | Evtl. nicht nötig in Rust |

### Query-Ausführung

| Methode | Beschreibung | Return |
|---------|--------------|--------|
| `executeQuery(sql, params, types, cache?)` | SELECT-Queries | `Result` |
| `executeStatement(sql, params, types)` | INSERT/UPDATE/DELETE | `int\|string` (affected rows) |
| `prepare(sql)` | Prepared Statement | `Statement` |

### Convenience-Methoden (Fetch)

```php
// Single Row
fetchAssociative(sql, params, types): array|false
fetchNumeric(sql, params, types): array|false
fetchOne(sql, params, types): mixed|false

// All Rows
fetchAllAssociative(sql, params, types): array
fetchAllNumeric(sql, params, types): array
fetchAllKeyValue(sql, params, types): array      // [col1 => col2, ...]
fetchAllAssociativeIndexed(sql, params, types): array  // [col1 => rest, ...]
fetchFirstColumn(sql, params, types): array

// Iterators (Memory-effizient)
iterateNumeric(sql, params, types): Traversable
iterateAssociative(sql, params, types): Traversable
iterateKeyValue(sql, params, types): Traversable
iterateColumn(sql, params, types): Traversable
```

### CRUD-Shortcuts

```php
// Einfache CRUD ohne QueryBuilder
insert(table, data, types): int|string
update(table, data, criteria, types): int|string
delete(table, criteria, types): int|string
```

### Transaktionen

```php
beginTransaction(): void
commit(): void
rollBack(): void
transactional(Closure $func): mixed  // Auto-commit/rollback

// Nesting-Info
getTransactionNestingLevel(): int
isTransactionActive(): bool

// Rollback-Only Flag
setRollbackOnly(): void
isRollbackOnly(): bool

// Savepoints (intern bei nesting > 1)
createSavepoint(name): void
releaseSavepoint(name): void
rollbackSavepoint(name): void
```

### Transaction Isolation

```php
setTransactionIsolation(TransactionIsolationLevel $level): void
getTransactionIsolation(): TransactionIsolationLevel
```

**TransactionIsolationLevel enum:**
- `READ_UNCOMMITTED`
- `READ_COMMITTED`
- `REPEATABLE_READ`
- `SERIALIZABLE`

### Factories

```php
createQueryBuilder(): QueryBuilder
createExpressionBuilder(): ExpressionBuilder
createSchemaManager(): AbstractSchemaManager
```

### Type-Konvertierung

```php
convertToDatabaseValue(value, typeName): mixed
convertToPHPValue(value, typeName): mixed
```

### Quoting

```php
quote(string): string              // String-Literal quoten (discouraged)
quoteSingleIdentifier(id): string  // Identifier quoten
```

## Wichtige Implementierungsdetails

### Lazy Connection

```php
protected function connect(): DriverConnection
{
    if ($this->_conn !== null) {
        return $this->_conn;
    }

    $connection = $this->_conn = $this->driver->connect($this->params);

    // Bei autoCommit=false: sofort Transaction starten
    if ($this->autoCommit === false) {
        $this->beginTransaction();
    }

    return $connection;
}
```

### Transaction Nesting mit Savepoints

```php
public function beginTransaction(): void
{
    $connection = $this->connect();
    ++$this->transactionNestingLevel;

    if ($this->transactionNestingLevel === 1) {
        $connection->beginTransaction();
    } else {
        // Nested: Savepoint statt echte Transaction
        $this->createSavepoint('DOCTRINE_' . $this->transactionNestingLevel);
    }
}
```

### Array-Parameter Expansion

```php
// SQL: "SELECT * FROM users WHERE id IN (?)"
// Params: [[1, 2, 3]]
// Types: [ArrayParameterType::INTEGER]
//
// Wird expandiert zu:
// SQL: "SELECT * FROM users WHERE id IN (?, ?, ?)"
// Params: [1, 2, 3]
```

### Parameter-Binding mit Type-Konvertierung

```php
private function bindParameters(DriverStatement $stmt, array $params, array $types): void
{
    foreach ($params as $key => $value) {
        if (isset($types[$key])) {
            $type = $types[$key];
            if ($type instanceof Type) {
                // DBAL Type: Konvertierung zu DB-Wert
                $value = $type->convertToDatabaseValue($value, $this->platform);
                $bindingType = $type->getBindingType();
            } else {
                // ParameterType enum direkt
                $bindingType = $type;
            }
        } else {
            $bindingType = ParameterType::STRING;
        }

        $stmt->bindValue($key, $value, $bindingType);
    }
}
```

## Rust-Portierung

### Struct-Design

```rust
pub struct Connection<D: Driver> {
    driver: D,
    conn: Option<D::Connection>,  // Lazy
    config: Configuration,
    params: ConnectionParams,

    auto_commit: bool,
    transaction_nesting: u32,
    transaction_isolation: Option<TransactionIsolationLevel>,
    is_rollback_only: bool,

    platform: OnceCell<Box<dyn Platform>>,
}
```

### Async-Consideration

```rust
impl<D: Driver> Connection<D> {
    pub async fn execute_query<'a>(
        &'a self,
        sql: &str,
        params: impl IntoParams,
    ) -> Result<impl Stream<Item = Result<Row>> + 'a, Error> {
        let conn = self.connect().await?;
        // ...
    }
}
```

### Builder-Pattern für Connection

```rust
let conn = Connection::builder()
    .driver(PostgresDriver::new())
    .host("localhost")
    .port(5432)
    .database("mydb")
    .username("user")
    .password("pass")
    .auto_commit(true)
    .build()
    .await?;
```
