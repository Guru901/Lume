# Lume Documentation

Welcome to Lume — a type-safe, ergonomic query builder and schema macro for MySQL.

- What is Lume: minimal, fast, compile-time safe layer for schema definition and queries
- Core concepts: `define_schema!`, typed `Column<T>`, `Query`, `Insert`, `Update`, filters
- Status: Actively evolving; see CHANGELOG for breaking changes

## Contents

- Getting Started: getting-started.md
- Defining Schemas: schema.md
- Selecting and Filtering: queries.md
- Inserts and Updates: inserts-updates.md
- Joins: joins.md
- Defaults and Auto-Increment: defaults-auto-increment.md
- Cookbook: cookbook.md

## Requirements

- Rust stable
- MySQL compatible database (tested with MySQL 8)
- Tokio async runtime

## Example

```rust
use lume::{database::Database, define_schema, schema::Schema};

define_schema! {
    Users {
        id: u64 [primary_key().not_null().auto_increment()],
        username: String [not_null()],
        email: String,
        is_active: bool [default_value(true)],
    }
}

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let db = Database::connect("mysql://user:pass@localhost/db").await?;

db.register_table::<Users>().await?;

db.insert(Users { id: None, username: "guru".into(), email: None, is_active: None })
    .execute()
    .await?;

let rows = db.query::<Users, SelectUsers>().execute().await?;
# Ok(())
# }
```
