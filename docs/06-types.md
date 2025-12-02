# Type System

Das Type-System verwaltet die bidirektionale Konvertierung zwischen PHP-Werten und Datenbank-Werten.

## Architektur

```
Type (abstract)
├── Konvertierung: PHP ↔ Database
├── SQL-Declaration generieren
├── Binding-Type für Prepared Statements
└── TypeRegistry für Lookup
```

## Eingebaute Types

| Type Name | PHP Type | SQL Declaration |
|-----------|----------|-----------------|
| `string` | `string` | VARCHAR |
| `ascii_string` | `string` | VARCHAR (ASCII) |
| `text` | `string` | TEXT/CLOB |
| `integer` | `int` | INT |
| `smallint` | `int` | SMALLINT |
| `bigint` | `string` (!) | BIGINT |
| `boolean` | `bool` | BOOLEAN/TINYINT |
| `decimal` | `string` | DECIMAL(p,s) |
| `float` | `float` | FLOAT/DOUBLE |
| `binary` | `resource\|string` | VARBINARY |
| `blob` | `resource\|string` | BLOB |
| `guid` | `string` | UUID/CHAR(36) |
| `date_mutable` | `DateTime` | DATE |
| `date_immutable` | `DateTimeImmutable` | DATE |
| `time_mutable` | `DateTime` | TIME |
| `time_immutable` | `DateTimeImmutable` | TIME |
| `datetime_mutable` | `DateTime` | DATETIME/TIMESTAMP |
| `datetime_immutable` | `DateTimeImmutable` | DATETIME/TIMESTAMP |
| `datetimetz_mutable` | `DateTime` | DATETIME (mit TZ) |
| `datetimetz_immutable` | `DateTimeImmutable` | DATETIME (mit TZ) |
| `dateinterval` | `DateInterval` | VARCHAR |
| `json` | `array\|object` | JSON/TEXT |
| `simple_array` | `array` | TEXT (comma-sep) |

## Type Interface

```php
abstract class Type
{
    /**
     * PHP-Wert → Database-Wert
     */
    public function convertToDatabaseValue(mixed $value, AbstractPlatform $platform): mixed
    {
        return $value;  // Default: keine Konvertierung
    }

    /**
     * Database-Wert → PHP-Wert
     */
    public function convertToPHPValue(mixed $value, AbstractPlatform $platform): mixed
    {
        return $value;  // Default: keine Konvertierung
    }

    /**
     * SQL-Declaration für Column
     */
    abstract public function getSQLDeclaration(array $column, AbstractPlatform $platform): string;

    /**
     * Binding-Type für Prepared Statements
     */
    public function getBindingType(): ParameterType
    {
        return ParameterType::STRING;  // Default
    }

    /**
     * SQL-Expression-Transformation (z.B. für spatial types)
     */
    public function convertToDatabaseValueSQL(string $sqlExpr, AbstractPlatform $platform): string
    {
        return $sqlExpr;
    }

    public function convertToPHPValueSQL(string $sqlExpr, AbstractPlatform $platform): string
    {
        return $sqlExpr;
    }
}
```

## Beispiel-Implementierungen

### DateTimeType

```php
class DateTimeType extends Type
{
    public function convertToDatabaseValue(mixed $value, AbstractPlatform $platform): ?string
    {
        if ($value === null) {
            return null;
        }

        if ($value instanceof DateTimeInterface) {
            return $value->format($platform->getDateTimeFormatString());
        }

        throw ConversionException::conversionFailedInvalidType($value, 'datetime');
    }

    public function convertToPHPValue(mixed $value, AbstractPlatform $platform): ?DateTime
    {
        if ($value === null || $value instanceof DateTime) {
            return $value;
        }

        $dateTime = DateTime::createFromFormat($platform->getDateTimeFormatString(), $value);

        if ($dateTime === false) {
            throw ConversionException::conversionFailed($value, 'datetime');
        }

        return $dateTime;
    }

    public function getSQLDeclaration(array $column, AbstractPlatform $platform): string
    {
        return $platform->getDateTimeTypeDeclarationSQL($column);
    }
}
```

### JsonType

```php
class JsonType extends Type
{
    public function convertToDatabaseValue(mixed $value, AbstractPlatform $platform): ?string
    {
        if ($value === null) {
            return null;
        }

        $encoded = json_encode($value, JSON_THROW_ON_ERROR | JSON_PRESERVE_ZERO_FRACTION);
        return $encoded;
    }

    public function convertToPHPValue(mixed $value, AbstractPlatform $platform): mixed
    {
        if ($value === null || $value === '') {
            return null;
        }

        if (is_resource($value)) {
            $value = stream_get_contents($value);
        }

        return json_decode($value, true, 512, JSON_THROW_ON_ERROR);
    }

    public function getSQLDeclaration(array $column, AbstractPlatform $platform): string
    {
        return $platform->getJsonTypeDeclarationSQL($column);
    }
}
```

### BooleanType

```php
class BooleanType extends Type
{
    public function convertToDatabaseValue(mixed $value, AbstractPlatform $platform): mixed
    {
        return $value === null ? null : $platform->convertBooleanToDatabaseValue($value);
    }

    public function convertToPHPValue(mixed $value, AbstractPlatform $platform): ?bool
    {
        return $value === null ? null : $platform->convertFromBoolean($value);
    }

    public function getBindingType(): ParameterType
    {
        return ParameterType::BOOLEAN;
    }

    public function getSQLDeclaration(array $column, AbstractPlatform $platform): string
    {
        return $platform->getBooleanTypeDeclarationSQL($column);
    }
}
```

### BigIntType

```php
class BigIntType extends Type
{
    // BIGINT wird als string zurückgegeben wegen PHP int Overflow!
    public function convertToPHPValue(mixed $value, AbstractPlatform $platform): ?string
    {
        return $value === null ? null : (string) $value;
    }

    public function getBindingType(): ParameterType
    {
        return ParameterType::STRING;  // Nicht INTEGER!
    }
}
```

## Type Registry

```php
// Type holen
$type = Type::getType('datetime_immutable');

// Prüfen ob existiert
if (Type::hasType('custom_type')) { ... }

// Custom Type registrieren
Type::addType('money', MoneyType::class);

// Bestehenden Type überschreiben
Type::overrideType('datetime', CustomDateTimeType::class);

// Alle Types
$map = Type::getTypesMap();  // ['string' => StringType::class, ...]
```

## Custom Types erstellen

```php
class MoneyType extends Type
{
    public function convertToDatabaseValue(mixed $value, AbstractPlatform $platform): ?int
    {
        if ($value === null) {
            return null;
        }

        // Money als Cents speichern
        return (int) ($value->getAmount() * 100);
    }

    public function convertToPHPValue(mixed $value, AbstractPlatform $platform): ?Money
    {
        if ($value === null) {
            return null;
        }

        return new Money($value / 100);
    }

    public function getSQLDeclaration(array $column, AbstractPlatform $platform): string
    {
        return $platform->getIntegerTypeDeclarationSQL($column);
    }

    public function getBindingType(): ParameterType
    {
        return ParameterType::INTEGER;
    }
}

// Registrieren
Type::addType('money', MoneyType::class);
```

## Verwendung

### In Queries

```php
$conn->executeQuery(
    'SELECT * FROM orders WHERE created_at > ?',
    [new DateTime('yesterday')],
    [Types::DATETIME_MUTABLE]
);
```

### In Schema

```php
$table->addColumn('price', Types::DECIMAL, [
    'precision' => 10,
    'scale' => 2,
]);

$table->addColumn('metadata', Types::JSON);
$table->addColumn('published_at', Types::DATETIME_IMMUTABLE);
```

### Direkte Konvertierung

```php
$dbValue = $conn->convertToDatabaseValue(new DateTime(), Types::DATETIME_MUTABLE);
$phpValue = $conn->convertToPHPValue('2024-01-15 10:30:00', Types::DATETIME_MUTABLE);
```

## Rust-Portierung

### Type Trait

```rust
pub trait SqlType: Send + Sync {
    /// Rust-Type → Database-Wert (als SqlValue)
    fn to_database(&self, value: &dyn Any, platform: &dyn Platform) -> Result<SqlValue, Error>;

    /// Database-Wert → Rust-Type
    fn to_rust(&self, value: SqlValue, platform: &dyn Platform) -> Result<Box<dyn Any>, Error>;

    /// SQL Declaration String
    fn sql_declaration(&self, column: &ColumnDefinition, platform: &dyn Platform) -> String;

    /// Binding-Type für Prepared Statements
    fn binding_type(&self) -> ParameterType {
        ParameterType::String
    }

    /// Type-Name
    fn name(&self) -> &'static str;
}
```

### Eingebaute Types

```rust
pub mod types {
    pub struct StringType;
    pub struct TextType;
    pub struct IntegerType;
    pub struct BigIntType;
    pub struct SmallIntType;
    pub struct BooleanType;
    pub struct FloatType;
    pub struct DecimalType;
    pub struct DateType;
    pub struct TimeType;
    pub struct DateTimeType;
    pub struct JsonType;
    pub struct BinaryType;
    pub struct BlobType;
    pub struct UuidType;
}

impl SqlType for DateTimeType {
    fn to_database(&self, value: &dyn Any, platform: &dyn Platform) -> Result<SqlValue, Error> {
        if let Some(dt) = value.downcast_ref::<DateTime<Utc>>() {
            let format = platform.datetime_format_string();
            Ok(SqlValue::String(dt.format(format).to_string()))
        } else if let Some(dt) = value.downcast_ref::<NaiveDateTime>() {
            let format = platform.datetime_format_string();
            Ok(SqlValue::String(dt.format(format).to_string()))
        } else {
            Err(Error::ConversionFailed("datetime"))
        }
    }

    fn to_rust(&self, value: SqlValue, platform: &dyn Platform) -> Result<Box<dyn Any>, Error> {
        match value {
            SqlValue::String(s) => {
                let format = platform.datetime_format_string();
                let dt = NaiveDateTime::parse_from_str(&s, format)?;
                Ok(Box::new(dt))
            }
            SqlValue::Null => Ok(Box::new(None::<NaiveDateTime>)),
            _ => Err(Error::ConversionFailed("datetime")),
        }
    }

    fn sql_declaration(&self, column: &ColumnDefinition, platform: &dyn Platform) -> String {
        platform.datetime_type_sql(column)
    }

    fn name(&self) -> &'static str { "datetime" }
}
```

### Type Registry

```rust
pub struct TypeRegistry {
    types: HashMap<&'static str, Box<dyn SqlType>>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self { types: HashMap::new() };

        // Builtin types registrieren
        registry.register("string", Box::new(StringType));
        registry.register("integer", Box::new(IntegerType));
        registry.register("boolean", Box::new(BooleanType));
        registry.register("datetime", Box::new(DateTimeType));
        registry.register("json", Box::new(JsonType));
        // ...

        registry
    }

    pub fn get(&self, name: &str) -> Option<&dyn SqlType> {
        self.types.get(name).map(|t| t.as_ref())
    }

    pub fn register(&mut self, name: &'static str, sql_type: Box<dyn SqlType>) {
        self.types.insert(name, sql_type);
    }
}

// Global registry (mit lazy_static oder OnceCell)
lazy_static! {
    static ref TYPE_REGISTRY: RwLock<TypeRegistry> = RwLock::new(TypeRegistry::new());
}
```

### Derive Macro für Custom Types

```rust
// Ziel: Automatische Type-Implementierung
#[derive(SqlType)]
#[sql_type(name = "money", sql = "INTEGER")]
struct Money {
    #[sql_convert(to_db = "cents", from_db = "from_cents")]
    cents: i64,
}

impl Money {
    fn cents(&self) -> i64 { self.cents }
    fn from_cents(cents: i64) -> Self { Self { cents } }
}
```

### SqlValue Enum

```rust
#[derive(Debug, Clone)]
pub enum SqlValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    // Für spezielle Typen
    Json(serde_json::Value),
    Uuid(uuid::Uuid),
    DateTime(chrono::NaiveDateTime),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
}

impl SqlValue {
    pub fn is_null(&self) -> bool {
        matches!(self, SqlValue::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            SqlValue::String(s) => Some(s),
            _ => None,
        }
    }

    // ... weitere Conversions
}
```
