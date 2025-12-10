# Advanced Topics

This guide covers advanced features and patterns in Lume.

## Raw SQL Queries

When you need to execute raw SQL that the query builder doesn't support:

```rust
let users = db
    .sql::<Users>("SELECT * FROM Users WHERE age > 18 AND status = 'active'")
    .await?;
```

**Warning**: Raw SQL bypasses type safety. Ensure your SQL matches the schema structure.

## Enums

Use Rust enums in your schemas:

```rust
use lume::{define_schema, enum_to_sql};

#[derive(Clone, Debug, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Banned,
}

define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        status: UserStatus [default_value(UserStatus::Active)],
    }
}

// Map enum variants to SQL values
enum_to_sql!(UserStatus {
    Active => "active",
    Inactive => "inactive",
    Banned => "banned",
});
```

## Generated Columns

Create computed columns (MySQL 5.7+, PostgreSQL):

```rust
define_schema! {
    Products {
        id: i32 [primary_key().not_null()],
        price: f64 [not_null()],
        tax_rate: f64 [not_null()],
        total_price: f64 [generated("ALWAYS AS (price * (1 + tax_rate)) STORED")],
    }
}
```

## Check Constraints

Add check constraints to validate data:

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        age: i32 [check("age >= 0 AND age <= 150")],
        email: String [check("email LIKE '%@%'")],
    }
}
```

## Column Comments

Add comments to columns (MySQL):

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null().comment("User's unique username")],
    }
}
```

## Character Sets and Collation

Specify character sets and collation (MySQL):

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null().charset("utf8mb4").collate("utf8mb4_unicode_ci")],
    }
}
```

## Invisible Columns

Create invisible columns (MySQL 8.0.23+):

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        internal_id: i32 [invisible()],
    }
}
```

## ON UPDATE CURRENT_TIMESTAMP

Automatically update timestamp columns:

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        updated_at: i64 [on_update_current_timestamp()],
    }
}
```

## Custom Default Values

Use SQL functions as defaults:

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        created_at: i64 [default_value(DefaultValueEnum::CurrentTimestamp)],
        uuid: String [default_value(DefaultValueEnum::Random)],
    }
}
```

## Table Information

Get information about registered tables:

```rust
// List all registered tables
let tables = Database::list_tables();
println!("Tables: {:?}", tables);

// Get column information for a table
if let Some(columns) = Database::get_table_info("Users") {
    for col in columns {
        println!("Column: {} ({})", col.name, col.data_type);
    }
}
```

## Migration SQL

Generate migration SQL without executing it:

```rust
let sql = Database::generate_migration_sql();
println!("{}", sql);
```

This prints the CREATE TABLE statements for all registered tables.

## Error Handling

Lume uses custom error types:

```rust
use lume::database::error::DatabaseError;

match db.query::<Users, SelectUsers>().execute().await {
    Ok(users) => {
        // Handle success
    }
    Err(DatabaseError::ConnectionError(e)) => {
        eprintln!("Connection error: {}", e);
    }
    Err(DatabaseError::QueryError(msg)) => {
        eprintln!("Query error: {}", msg);
    }
    Err(DatabaseError::ExecutionError(msg)) => {
        eprintln!("Execution error: {}", msg);
    }
}
```

## Type Conversions

Lume automatically converts between Rust types and SQL values:

```rust
use lume::schema::{Value, convert_to_value};

// Convert Rust values to SQL values
let int_val: Value = convert_to_value(42);
let str_val: Value = convert_to_value("hello".to_string());
let bool_val: Value = convert_to_value(true);
```

## Custom SQL Types

Implement `CustomSqlType` for custom types:

```rust
use lume::schema::CustomSqlType;

#[derive(Clone, Debug)]
struct MyCustomType(String);

impl CustomSqlType for MyCustomType {}
impl ToString for MyCustomType {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        custom_field: MyCustomType,
    }
}
```

## Validators

Add runtime validators to columns (if supported):

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        email: String [validate(|v| v.contains("@"))],
    }
}
```

## Best Practices

1. **Use enums for status fields** - More type-safe than strings
2. **Add constraints** - Use check constraints for data validation
3. **Index foreign keys** - Always index columns used in joins
4. **Use appropriate types** - Choose the right integer size
5. **Handle errors properly** - Always handle `Result` types
6. **Use raw SQL sparingly** - Prefer the query builder when possible
7. **Document schemas** - Use comments to document your schema

## Performance Tips

1. **Index frequently queried columns** - Use `indexed()` constraint
2. **Select only needed columns** - Use `.select()` in queries
3. **Use bulk operations** - Use `insert_many()` for multiple inserts
4. **Filter early** - Apply filters before joins
5. **Use limits** - Always use `.limit()` when appropriate

## Database-Specific Features

### MySQL

- `AUTO_INCREMENT` for integer primary keys
- `CHARACTER SET` and `COLLATE` for text columns
- `COMMENT` for column documentation
- `INVISIBLE` columns
- `ON UPDATE CURRENT_TIMESTAMP`

### PostgreSQL

- `SERIAL` and `BIGSERIAL` types
- `ILIKE` for case-insensitive pattern matching
- Array types (may require custom handling)
- JSON/JSONB types (may require custom handling)

### SQLite

- Simpler type system (affinity-based)
- No `AUTO_INCREMENT` (use `INTEGER PRIMARY KEY`)
- Limited constraint support compared to MySQL/PostgreSQL

## Next Steps

- Review the [Schema Definition](schema.md) guide
- Check out [Queries](queries.md) for query building
- Explore [Filters](filters.md) for filtering options
