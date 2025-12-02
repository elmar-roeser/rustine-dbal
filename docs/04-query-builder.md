# QueryBuilder

Der QueryBuilder ermöglicht die programmatische Konstruktion von SQL-Queries mit einem Fluent-Interface.

## Query-Typen

```php
enum QueryType
{
    case SELECT;
    case INSERT;
    case UPDATE;
    case DELETE;
    case UNION;
}
```

## Zustand

```php
class QueryBuilder
{
    private ?string $sql = null;              // Cached SQL
    private array $params = [];               // Positional oder Named
    private array $types = [];                // Parameter-Typen
    private QueryType $type = QueryType::SELECT;

    private int $firstResult = 0;             // OFFSET
    private ?int $maxResults = null;          // LIMIT

    // SELECT-spezifisch
    private array $select = [];
    private bool $distinct = false;
    private array $from = [];                 // From[]
    private array $join = [];                 // [alias => Join[]]
    private string|CompositeExpression|null $where = null;
    private array $groupBy = [];
    private string|CompositeExpression|null $having = null;
    private array $orderBy = [];
    private ?ForUpdate $forUpdate = null;

    // INSERT-spezifisch
    private ?string $table = null;
    private array $values = [];

    // UPDATE-spezifisch
    private array $set = [];

    // UNION
    private array $unionParts = [];           // Union[]

    // CTE
    private array $commonTableExpressions = [];
}
```

## SELECT-Queries

### Basis-Syntax

```php
$qb = $conn->createQueryBuilder();

$qb->select('u.id', 'u.name', 'u.email')
   ->from('users', 'u')
   ->where('u.active = :active')
   ->orderBy('u.name', 'ASC')
   ->setParameter('active', true)
   ->setMaxResults(10);

$result = $qb->executeQuery();
```

### SELECT-Methoden

```php
// Spalten auswählen
select(string ...$columns): self
addSelect(string ...$columns): self
distinct(): self

// FROM
from(string $table, ?string $alias = null): self

// JOINs
join(string $fromAlias, string $table, string $alias, ?string $condition = null): self
innerJoin(string $fromAlias, string $table, string $alias, ?string $condition = null): self
leftJoin(string $fromAlias, string $table, string $alias, ?string $condition = null): self
rightJoin(string $fromAlias, string $table, string $alias, ?string $condition = null): self

// WHERE
where(string|CompositeExpression $predicate): self
andWhere(string|CompositeExpression $predicate): self
orWhere(string|CompositeExpression $predicate): self

// GROUP BY / HAVING
groupBy(string ...$expressions): self
addGroupBy(string ...$expressions): self
having(string|CompositeExpression $predicate): self
andHaving(string|CompositeExpression $predicate): self
orHaving(string|CompositeExpression $predicate): self

// ORDER BY
orderBy(string $sort, ?string $order = null): self
addOrderBy(string $sort, ?string $order = null): self

// LIMIT / OFFSET
setMaxResults(?int $max): self
setFirstResult(int $first): self

// FOR UPDATE (Locking)
forUpdate(ConflictResolutionMode $mode = ConflictResolutionMode::ORDINARY): self
```

### UNION

```php
$qb1 = $conn->createQueryBuilder()->select('id')->from('users');
$qb2 = $conn->createQueryBuilder()->select('id')->from('admins');

$qb = $conn->createQueryBuilder()
    ->union($qb1)
    ->addUnion($qb2, UnionType::ALL);
```

### CTE (Common Table Expressions)

```php
$cte = $conn->createQueryBuilder()
    ->select('id', 'parent_id', 'name')
    ->from('categories')
    ->where('parent_id IS NULL');

$qb = $conn->createQueryBuilder()
    ->with('roots', $cte)
    ->select('*')
    ->from('roots');
```

## INSERT-Queries

```php
$qb = $conn->createQueryBuilder();

$qb->insert('users')
   ->values([
       'name' => ':name',
       'email' => ':email',
       'created_at' => ':created',
   ])
   ->setParameter('name', 'John')
   ->setParameter('email', 'john@example.com')
   ->setParameter('created', new DateTime(), Types::DATETIME_MUTABLE);

$affectedRows = $qb->executeStatement();
```

### INSERT-Methoden

```php
insert(string $table): self
values(array $values): self
setValue(string $column, string $value): self
```

## UPDATE-Queries

```php
$qb = $conn->createQueryBuilder();

$qb->update('users', 'u')
   ->set('u.name', ':name')
   ->set('u.updated_at', ':updated')
   ->where('u.id = :id')
   ->setParameter('name', 'Jane')
   ->setParameter('updated', new DateTime(), Types::DATETIME_MUTABLE)
   ->setParameter('id', 42);

$affectedRows = $qb->executeStatement();
```

### UPDATE-Methoden

```php
update(string $table, ?string $alias = null): self
set(string $column, string $value): self
```

## DELETE-Queries

```php
$qb = $conn->createQueryBuilder();

$qb->delete('users', 'u')
   ->where('u.active = :active')
   ->andWhere('u.last_login < :date')
   ->setParameter('active', false)
   ->setParameter('date', new DateTime('-1 year'), Types::DATETIME_MUTABLE);

$affectedRows = $qb->executeStatement();
```

### DELETE-Methoden

```php
delete(string $table, ?string $alias = null): self
```

## Parameter-Binding

### Positional Parameters

```php
$qb->select('*')
   ->from('users')
   ->where('id = ?')
   ->setParameter(0, 42);
```

### Named Parameters

```php
$qb->select('*')
   ->from('users')
   ->where('id = :id')
   ->setParameter('id', 42);
```

### Mit Type-Hints

```php
$qb->setParameter('date', new DateTime(), Types::DATETIME_MUTABLE);
$qb->setParameter('active', true, ParameterType::BOOLEAN);
$qb->setParameter('ids', [1, 2, 3], ArrayParameterType::INTEGER);
```

### Alle Parameter setzen

```php
$qb->setParameters([
    'name' => 'John',
    'active' => true,
], [
    'name' => ParameterType::STRING,
    'active' => ParameterType::BOOLEAN,
]);
```

## ExpressionBuilder

```php
$expr = $qb->expr();

// Vergleiche
$expr->eq('u.id', ':id')           // u.id = :id
$expr->neq('u.status', ':status')  // u.status <> :status
$expr->lt('u.age', ':age')         // u.age < :age
$expr->lte('u.age', ':age')        // u.age <= :age
$expr->gt('u.age', ':age')         // u.age > :age
$expr->gte('u.age', ':age')        // u.age >= :age

// NULL
$expr->isNull('u.deleted_at')      // u.deleted_at IS NULL
$expr->isNotNull('u.email')        // u.email IS NOT NULL

// IN
$expr->in('u.status', [':s1', ':s2'])     // u.status IN (:s1, :s2)
$expr->notIn('u.role', [':r1', ':r2'])    // u.role NOT IN (:r1, :r2)

// LIKE
$expr->like('u.name', ':pattern')         // u.name LIKE :pattern
$expr->notLike('u.email', ':pattern')     // u.email NOT LIKE :pattern

// BETWEEN (nicht direkt, aber konstruierbar)

// Komposition
$expr->and($expr->eq(...), $expr->gt(...))  // (... AND ...)
$expr->or($expr->eq(...), $expr->eq(...))   // (... OR ...)
```

### Beispiel: Komplexe Bedingungen

```php
$qb->where(
    $expr->and(
        $expr->eq('u.active', ':active'),
        $expr->or(
            $expr->like('u.name', ':name'),
            $expr->like('u.email', ':email')
        )
    )
);
// WHERE (u.active = :active AND (u.name LIKE :name OR u.email LIKE :email))
```

## Ausführung

```php
// SELECT → Result
$result = $qb->executeQuery();

// Convenience-Methoden
$row = $qb->fetchAssociative();        // Erste Zeile als assoc array
$rows = $qb->fetchAllAssociative();    // Alle Zeilen
$value = $qb->fetchOne();              // Erster Wert der ersten Zeile
$column = $qb->fetchFirstColumn();     // Erste Spalte aller Zeilen

// INSERT/UPDATE/DELETE → int (affected rows)
$count = $qb->executeStatement();
```

## SQL abrufen

```php
$sql = $qb->getSQL();
$params = $qb->getParameters();
$types = $qb->getParameterTypes();
```

## Rust-Portierung

### Struct-Design

```rust
pub struct QueryBuilder<'conn> {
    connection: &'conn Connection,
    sql: Option<String>,  // Cached

    query_type: QueryType,
    params: Parameters,

    // SELECT
    select: Vec<String>,
    distinct: bool,
    from: Vec<From>,
    joins: HashMap<String, Vec<Join>>,
    where_clause: Option<Expression>,
    group_by: Vec<String>,
    having: Option<Expression>,
    order_by: Vec<(String, Order)>,
    limit: Option<u64>,
    offset: u64,
    for_update: Option<ForUpdate>,

    // INSERT/UPDATE
    table: Option<String>,
    values: HashMap<String, String>,
    set: Vec<(String, String)>,

    // UNION
    unions: Vec<Union>,
    ctes: Vec<Cte>,
}
```

### Builder-Pattern mit Method Chaining

```rust
impl<'conn> QueryBuilder<'conn> {
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query_type = QueryType::Select;
        self.select = columns.iter().map(|s| s.to_string()).collect();
        self.invalidate_sql();
        self
    }

    pub fn from(mut self, table: &str, alias: Option<&str>) -> Self {
        self.from.push(From { table: table.into(), alias: alias.map(Into::into) });
        self.invalidate_sql();
        self
    }

    pub fn where_clause(mut self, expr: impl Into<Expression>) -> Self {
        self.where_clause = Some(expr.into());
        self.invalidate_sql();
        self
    }

    pub fn and_where(mut self, expr: impl Into<Expression>) -> Self {
        self.where_clause = Some(match self.where_clause.take() {
            Some(existing) => Expression::And(Box::new(existing), Box::new(expr.into())),
            None => expr.into(),
        });
        self.invalidate_sql();
        self
    }

    pub fn set_parameter<V: ToSql>(mut self, key: impl Into<ParamKey>, value: V) -> Self {
        self.params.set(key.into(), value);
        self
    }

    fn invalidate_sql(&mut self) {
        self.sql = None;
    }
}
```

### Expression-Builder

```rust
pub struct ExpressionBuilder;

impl ExpressionBuilder {
    pub fn eq(left: &str, right: &str) -> Expression {
        Expression::Comparison(left.into(), "=".into(), right.into())
    }

    pub fn neq(left: &str, right: &str) -> Expression {
        Expression::Comparison(left.into(), "<>".into(), right.into())
    }

    pub fn is_null(column: &str) -> Expression {
        Expression::IsNull(column.into())
    }

    pub fn and(exprs: Vec<Expression>) -> Expression {
        Expression::Composite(CompositeType::And, exprs)
    }

    pub fn or(exprs: Vec<Expression>) -> Expression {
        Expression::Composite(CompositeType::Or, exprs)
    }
}

// Usage
let qb = conn.query_builder()
    .select(&["id", "name"])
    .from("users", Some("u"))
    .where_clause(expr::and(vec![
        expr::eq("u.active", ":active"),
        expr::gt("u.age", ":min_age"),
    ]))
    .set_parameter("active", true)
    .set_parameter("min_age", 18);
```

### Async Execution

```rust
impl<'conn> QueryBuilder<'conn> {
    pub async fn execute_query(&self) -> Result<impl Stream<Item = Result<Row>>, Error> {
        let sql = self.get_sql()?;
        self.connection.execute_query(&sql, &self.params).await
    }

    pub async fn execute_statement(&self) -> Result<u64, Error> {
        let sql = self.get_sql()?;
        self.connection.execute_statement(&sql, &self.params).await
    }

    pub async fn fetch_one(&self) -> Result<Option<Row>, Error> {
        let mut stream = self.execute_query().await?;
        stream.next().await.transpose()
    }

    pub async fn fetch_all(&self) -> Result<Vec<Row>, Error> {
        self.execute_query().await?.try_collect().await
    }
}
```
