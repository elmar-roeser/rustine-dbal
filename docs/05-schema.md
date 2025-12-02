# Schema Management

Das Schema-System ermöglicht Datenbank-Introspection und Schema-Manipulation.

## Klassen-Übersicht

### Schema-Objekte

```
Schema
├── Table[]
│   ├── Column[]
│   ├── Index[]
│   ├── ForeignKeyConstraint[]
│   ├── UniqueConstraint[]
│   └── PrimaryKeyConstraint
├── Sequence[]
└── View[]
```

### Manager-Hierarchie

```
AbstractSchemaManager
├── MySQLSchemaManager
├── PostgreSQLSchemaManager
├── SQLiteSchemaManager
├── OracleSchemaManager
├── SQLServerSchemaManager
└── DB2SchemaManager
```

## SchemaManager API

### Listen/Introspection

```php
// Datenbanken
listDatabases(): array<int, string>

// Schemas (PostgreSQL, SQL Server)
listSchemaNames(): array<int, string>

// Tabellen
listTableNames(): array<int, string>
listTables(): array<int, Table>
introspectTable(string $name): Table

// Spalten
listTableColumns(string $table): array<string, Column>

// Indexes
listTableIndexes(string $table): array<string, Index>

// Foreign Keys
listTableForeignKeys(string $table): array<int, ForeignKeyConstraint>

// Sequences
listSequences(): array<int, Sequence>

// Views
listViews(): array<string, View>
```

### Schema-Objekt holen

```php
// Komplettes Schema der Datenbank
introspectSchema(): Schema

// Einzelne Tabelle
introspectTable(string $tableName): Table
```

### Existenz prüfen

```php
tablesExist(array $names): bool
tableExists(string $name): bool
```

### DDL-Operationen

```php
// Tabellen
createTable(Table $table): void
dropTable(string $table): void
renameTable(string $oldName, string $newName): void

// Spalten
alterTable(TableDiff $diff): void

// Indexes
createIndex(Index $index, string $table): void
dropIndex(string $index, string $table): void

// Foreign Keys
createForeignKey(ForeignKeyConstraint $fk, string $table): void
dropForeignKey(string|ForeignKeyConstraint $fk, string $table): void

// Sequences
createSequence(Sequence $sequence): void
dropSequence(string $sequence): void

// Views
createView(View $view): void
dropView(string $name): void

// Datenbanken
createDatabase(string $database): void
dropDatabase(string $database): void
```

## Schema-Objekte

### Table

```php
class Table
{
    private string $name;
    private array $columns = [];           // Column[]
    private array $indexes = [];           // Index[]
    private ?PrimaryKeyConstraint $primaryKey = null;
    private array $uniqueConstraints = []; // UniqueConstraint[]
    private array $foreignKeys = [];       // ForeignKeyConstraint[]
    private array $options = [];           // Engine, Collation, etc.

    // Factory-Methoden
    public function addColumn(string $name, string $typeName, array $options = []): Column;
    public function modifyColumn(string $name, array $options): Column;
    public function dropColumn(string $name): self;
    public function renameColumn(string $oldName, string $newName): Column;

    public function setPrimaryKey(array $columns, ?string $name = null): self;
    public function addIndex(array $columns, ?string $name = null, array $flags = []): self;
    public function addUniqueConstraint(array $columns, ?string $name = null): self;
    public function addForeignKeyConstraint(
        string $foreignTable,
        array $localColumns,
        array $foreignColumns,
        array $options = [],
        ?string $name = null
    ): self;

    // Getter
    public function getName(): string;
    public function getColumns(): array;
    public function getColumn(string $name): Column;
    public function hasColumn(string $name): bool;
    public function getPrimaryKey(): ?PrimaryKeyConstraint;
    public function getIndexes(): array;
    public function getForeignKeys(): array;
}
```

### Column

```php
class Column
{
    private string $name;
    private Type $type;
    private ?int $length = null;
    private ?int $precision = null;
    private ?int $scale = null;
    private bool $unsigned = false;
    private bool $fixed = false;
    private bool $notnull = true;
    private mixed $default = null;
    private bool $autoincrement = false;
    private array $platformOptions = [];
    private ?string $comment = null;

    // Getters/Setters für alle Properties
    public function getName(): string;
    public function getType(): Type;
    public function getLength(): ?int;
    public function getPrecision(): ?int;
    public function getScale(): ?int;
    public function getUnsigned(): bool;
    public function getFixed(): bool;
    public function getNotnull(): bool;
    public function getDefault(): mixed;
    public function getAutoincrement(): bool;
    public function getComment(): ?string;
}
```

### Index

```php
class Index
{
    private ?string $name;
    private array $columns;       // Spalten-Namen
    private bool $isUnique;
    private bool $isPrimary;
    private array $flags = [];    // FULLTEXT, SPATIAL, etc.
    private array $options = [];  // lengths, etc.

    public function getName(): ?string;
    public function getColumns(): array;
    public function isUnique(): bool;
    public function isPrimary(): bool;
    public function hasFlag(string $flag): bool;
}
```

### ForeignKeyConstraint

```php
class ForeignKeyConstraint
{
    private ?string $name;
    private array $localColumns;
    private string $foreignTableName;
    private array $foreignColumns;
    private array $options = [];  // onDelete, onUpdate

    public function getName(): ?string;
    public function getLocalColumns(): array;
    public function getForeignTableName(): string;
    public function getForeignColumns(): array;
    public function getOption(string $name): mixed;  // ON DELETE, ON UPDATE
    public function onDelete(): ?string;  // CASCADE, SET NULL, etc.
    public function onUpdate(): ?string;
}
```

### Sequence

```php
class Sequence
{
    private string $name;
    private int $allocationSize = 1;   // INCREMENT BY
    private int $initialValue = 1;     // START WITH
    private ?int $cache = null;

    public function getName(): string;
    public function getAllocationSize(): int;
    public function getInitialValue(): int;
}
```

## Schema-Vergleich & Diff

### Comparator

```php
class Comparator
{
    public function compareSchemas(Schema $from, Schema $to): SchemaDiff;
    public function compareTables(Table $from, Table $to): TableDiff;
}
```

### SchemaDiff

```php
class SchemaDiff
{
    public readonly array $createdTables;      // Table[]
    public readonly array $alteredTables;      // TableDiff[]
    public readonly array $droppedTables;      // Table[]
    public readonly array $createdSequences;   // Sequence[]
    public readonly array $alteredSequences;   // Sequence[]
    public readonly array $droppedSequences;   // Sequence[]
}
```

### TableDiff

```php
class TableDiff
{
    public readonly Table $oldTable;

    public readonly array $addedColumns;       // Column[]
    public readonly array $modifiedColumns;    // ColumnDiff[]
    public readonly array $droppedColumns;     // Column[]
    public readonly array $renamedColumns;     // [oldName => Column]

    public readonly array $addedIndexes;       // Index[]
    public readonly array $modifiedIndexes;    // Index[]
    public readonly array $droppedIndexes;     // Index[]
    public readonly array $renamedIndexes;     // [oldName => Index]

    public readonly array $addedForeignKeys;   // ForeignKeyConstraint[]
    public readonly array $modifiedForeignKeys;// ForeignKeyConstraint[]
    public readonly array $droppedForeignKeys; // ForeignKeyConstraint[]
}
```

## Schema programmatisch erstellen

```php
$schema = new Schema();

$users = $schema->createTable('users');
$users->addColumn('id', Types::INTEGER, ['autoincrement' => true]);
$users->addColumn('email', Types::STRING, ['length' => 255]);
$users->addColumn('password', Types::STRING, ['length' => 255]);
$users->addColumn('created_at', Types::DATETIME_IMMUTABLE);
$users->addColumn('is_active', Types::BOOLEAN, ['default' => true]);
$users->setPrimaryKey(['id']);
$users->addUniqueConstraint(['email'], 'users_email_unique');

$posts = $schema->createTable('posts');
$posts->addColumn('id', Types::INTEGER, ['autoincrement' => true]);
$posts->addColumn('user_id', Types::INTEGER);
$posts->addColumn('title', Types::STRING, ['length' => 255]);
$posts->addColumn('content', Types::TEXT);
$posts->setPrimaryKey(['id']);
$posts->addIndex(['user_id'], 'posts_user_idx');
$posts->addForeignKeyConstraint(
    'users',
    ['user_id'],
    ['id'],
    ['onDelete' => 'CASCADE'],
    'posts_user_fk'
);
```

## Migration-SQL generieren

```php
// Aktuelles Schema holen
$fromSchema = $schemaManager->introspectSchema();

// Gewünschtes Schema definieren
$toSchema = new Schema();
// ... Tabellen hinzufügen

// Diff berechnen
$comparator = new Comparator();
$diff = $comparator->compareSchemas($fromSchema, $toSchema);

// SQL generieren
$platform = $connection->getDatabasePlatform();
$sql = $platform->getAlterSchemaSQL($diff);

// Ausführen
foreach ($sql as $statement) {
    $connection->executeStatement($statement);
}
```

## Rust-Portierung

### Schema-Structs

```rust
pub struct Schema {
    pub name: Option<String>,
    pub tables: HashMap<String, Table>,
    pub sequences: HashMap<String, Sequence>,
    pub views: HashMap<String, View>,
}

pub struct Table {
    pub name: String,
    pub columns: IndexMap<String, Column>,  // Reihenfolge wichtig
    pub primary_key: Option<PrimaryKeyConstraint>,
    pub indexes: HashMap<String, Index>,
    pub unique_constraints: HashMap<String, UniqueConstraint>,
    pub foreign_keys: HashMap<String, ForeignKeyConstraint>,
    pub options: TableOptions,
}

pub struct Column {
    pub name: String,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub default: Option<DefaultValue>,
    pub auto_increment: bool,
    pub comment: Option<String>,
    pub platform_options: HashMap<String, Value>,
}

pub enum ColumnType {
    Integer { unsigned: bool },
    BigInt { unsigned: bool },
    SmallInt { unsigned: bool },
    String { length: Option<u32>, fixed: bool },
    Text,
    Binary { length: Option<u32>, fixed: bool },
    Blob,
    Boolean,
    Decimal { precision: u32, scale: u32 },
    Float,
    Date,
    Time,
    DateTime { timezone: bool },
    Json { binary: bool },
    Uuid,
    Custom(String),
}

pub struct Index {
    pub name: Option<String>,
    pub columns: Vec<IndexColumn>,
    pub unique: bool,
    pub flags: HashSet<IndexFlag>,
}

pub struct ForeignKeyConstraint {
    pub name: Option<String>,
    pub local_columns: Vec<String>,
    pub foreign_table: String,
    pub foreign_columns: Vec<String>,
    pub on_delete: Option<ReferentialAction>,
    pub on_update: Option<ReferentialAction>,
}

pub enum ReferentialAction {
    Cascade,
    SetNull,
    SetDefault,
    Restrict,
    NoAction,
}
```

### SchemaManager Trait

```rust
#[async_trait]
pub trait SchemaManager: Send + Sync {
    // Introspection
    async fn list_databases(&self) -> Result<Vec<String>, Error>;
    async fn list_table_names(&self) -> Result<Vec<String>, Error>;
    async fn list_tables(&self) -> Result<Vec<Table>, Error>;
    async fn introspect_table(&self, name: &str) -> Result<Table, Error>;
    async fn introspect_schema(&self) -> Result<Schema, Error>;

    // Existence checks
    async fn table_exists(&self, name: &str) -> Result<bool, Error>;

    // DDL operations
    async fn create_table(&self, table: &Table) -> Result<(), Error>;
    async fn drop_table(&self, name: &str) -> Result<(), Error>;
    async fn alter_table(&self, diff: &TableDiff) -> Result<(), Error>;

    async fn create_index(&self, index: &Index, table: &str) -> Result<(), Error>;
    async fn drop_index(&self, name: &str, table: &str) -> Result<(), Error>;

    async fn create_foreign_key(&self, fk: &ForeignKeyConstraint, table: &str) -> Result<(), Error>;
    async fn drop_foreign_key(&self, name: &str, table: &str) -> Result<(), Error>;
}
```

### Schema-Diff

```rust
pub struct SchemaDiff {
    pub created_tables: Vec<Table>,
    pub altered_tables: Vec<TableDiff>,
    pub dropped_tables: Vec<String>,
    pub created_sequences: Vec<Sequence>,
    pub dropped_sequences: Vec<String>,
}

pub struct TableDiff {
    pub table_name: String,
    pub added_columns: Vec<Column>,
    pub modified_columns: Vec<ColumnDiff>,
    pub dropped_columns: Vec<String>,
    pub renamed_columns: Vec<(String, String)>,
    pub added_indexes: Vec<Index>,
    pub dropped_indexes: Vec<String>,
    pub added_foreign_keys: Vec<ForeignKeyConstraint>,
    pub dropped_foreign_keys: Vec<String>,
}

pub struct Comparator;

impl Comparator {
    pub fn compare_schemas(from: &Schema, to: &Schema) -> SchemaDiff {
        // ...
    }

    pub fn compare_tables(from: &Table, to: &Table) -> TableDiff {
        // ...
    }
}
```

### Builder-Pattern für Table

```rust
let users = Table::builder("users")
    .add_column(Column::integer("id").auto_increment())
    .add_column(Column::string("email", 255).not_null())
    .add_column(Column::string("password", 255).not_null())
    .add_column(Column::datetime("created_at"))
    .add_column(Column::boolean("is_active").default(true))
    .primary_key(&["id"])
    .unique_constraint(&["email"], "users_email_unique")
    .build();
```
