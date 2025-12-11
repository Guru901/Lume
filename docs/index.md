# Lume Documentation

Welcome to the Lume documentation! Lume is a type-safe, ergonomic query builder and ORM for SQL databases.

## What is Lume?

Lume is a minimal, fast, compile-time safe layer for schema definition and database queries. It provides:

- **Type-safe queries** - Compile-time checking prevents runtime errors
- **Ergonomic API** - Clean, intuitive interface inspired by modern ORMs
- **Multi-database support** - Works with MySQL, PostgreSQL, and SQLite
- **Zero-cost abstractions** - Minimal runtime overhead
- **SQL injection protection** - Parameterized queries by default

## Documentation Structure

### Getting Started

- [Getting Started Guide](getting-started.md) - Installation, setup, and your first query

### Core Concepts

- [Schema Definition](schema.md) - Define your database schemas with the `define_schema!` macro
- [Queries](queries.md) - Build and execute type-safe queries
- [Filters](filters.md) - Filter data with conditions and operators
- [Inserts & Updates](inserts-updates.md) - Insert and update records
- [Joins](joins.md) - Join multiple tables in queries

### Advanced Topics

- [Advanced Features](advanced.md) - Raw SQL, transactions, enums, and more

## Quick Example

```rust
use lume::{database::Database, define_schema, filter::eq_value};

define_schema! {
    Users {
        id: Uuid [primary_key().not_null().default_random()],
        username: String [not_null()],
        email: String,
        age: i32,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect("mysql://user:pass@localhost/db").await?;
    db.register_table::<Users>().await?;

    // Insert
    db.insert(Users {
        id: None,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: Some(30),
    })
    .execute()
    .await?;

    // Query
    let users = db
        .query::<Users, SelectUsers>()
        .filter(eq_value(Users::username(), "alice"))
        .execute()
        .await?;

    Ok(())
}
```

## Requirements

- Rust stable (2021 edition or later)
- Tokio async runtime
- MySQL 8+, PostgreSQL 12+, or SQLite 3.8+

## Next Steps

Start with the [Getting Started Guide](getting-started.md) to set up Lume in your project.
