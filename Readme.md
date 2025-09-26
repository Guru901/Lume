# Lume

A type-safe, ergonomic schema builder and ORM for SQL databases, inspired by Drizzle ORM.

[![Crates.io](https://img.shields.io/crates/v/lume)](https://crates.io/crates/lume)
[![Documentation](https://img.shields.io/docsrs/lume)](https://docs.rs/lume)
[![License](https://img.shields.io/crates/l/lume)](LICENSE)

## Features

- 🚀 **Type-safe**: Compile-time type checking for all database operations
- 🎯 **Ergonomic**: Clean, intuitive API inspired by modern ORMs
- ⚡ **Performance**: Zero-cost abstractions with minimal runtime overhead
- 🔧 **Flexible**: Support for various column constraints and SQL types
- 🛡️ **Safe**: Prevents SQL injection and runtime type errors
- 📦 **Lightweight**: Minimal dependencies, maximum functionality

## Quick Start

Add Lume to your `Cargo.toml`:

```toml
[dependencies]
lume = "0.1"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### Basic Usage

```rust
use lume::{
    database::Database,
    define_schema,
    filter::eq,
    schema::{ColumnInfo, Schema},
};

// Define your database schema
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        email: String,
        age: i32,
        is_active: bool [default_value(true)],
        created_at: i64 [not_null()],
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to your MySQL database
    let db = Database::connect("mysql://user:password@localhost/database").await?;

    // Create tables (if they don't exist)
    Users::ensure_registered();

    // Type-safe queries
    let users = db
        .query::<Users>()
        .filter(eq(Users::username(), "john_doe"))
        .execute()
        .await?;

    for user in users {
        let username: Option<String> = user.get(Users::username());
        let age: Option<i32> = user.get(Users::age());
        println!(
            "User: {} (age: {})",
            username.unwrap_or_default(),
            age.unwrap_or(0)
        );
    }

    db.insert(Users {
        id: 0,
        age: 25,
        username: "john_doe".to_string(),
        email: "john.doe@example.com".to_string(),
        is_active: true,
        created_at: 1677721600,
    })
    .execute()
    .await
    .unwrap();

    Ok(())
}
```

## Schema Definition

Lume provides a powerful macro system for defining database schemas:

```rust
define_schema! {
    Product {
        id: i32 [primary_key().not_null()],
        name: String [not_null().unique()],
        description: String,
        price: f64 [not_null()],
        stock: i32 [default_value(0)],
        category_id: i32 [not_null().indexed()],
        created_at: i64 [not_null()],
        updated_at: i64 [not_null()],
    }
}
```

### Supported Types

- `String` → `VARCHAR(255)`
- `i32` → `INTEGER`
- `i64` → `BIGINT`
- `f32` → `FLOAT`
- `f64` → `DOUBLE`
- `bool` → `BOOLEAN`

### Column Constraints

- `primary_key()` - Sets the column as primary key
- `not_null()` - Makes the column NOT NULL
- `unique()` - Adds a UNIQUE constraint
- `indexed()` - Creates an index on the column
- `default_value(value)` - Sets a default value

## Type-Safe Queries

```rust
// Filter by equality
let active_users = db
    .query::<Users>()
    .filter(Filter::eq("is_active", true))
    .execute()
    .await?;

// Multiple filters
let young_active_users = db
    .query::<Users>()
    .filter(Filter::eq("is_active", true))
    .filter(Filter::lt("age", 25))
    .execute()
    .await?;

// Access row data type-safely
for user in young_active_users {
    let id: Option<i32> = user.get(Users::id());
    let username: Option<String> = user.get(Users::username());
    let email: Option<String> = user.get(Users::email());

    println!("User {}: {} ({})", id.unwrap_or(0), username.unwrap_or_default(), email.unwrap_or_default());
}
```

## Advanced Features

### Custom Default Values

```rust
define_schema! {
    Settings {
        id: i32 [primary_key().not_null()],
        theme: String [default_value("dark".to_string())],
        notifications: bool [default_value(true)],
        max_connections: i32 [default_value(10)],
    }
}
```

### Table Relationships

```rust
define_schema! {
    Author {
        id: i32 [primary_key().not_null()],
        name: String [not_null()],
        email: String [not_null().unique()],
    }

    Book {
        id: i32 [primary_key().not_null()],
        title: String [not_null()],
        author_id: i32 [not_null()], // References Author.id
        published_year: i32,
        isbn: String [unique()],
    }
}
```

## Error Handling

Lume provides comprehensive error handling:

```rust
use lume::database::DatabaseError;

match db.query::<Users>().execute().await {
    Ok(users) => {
        println!("Found {} users", users.len());
    }
    Err(DatabaseError::ConnectionFailed) => {
        eprintln!("Failed to connect to database");
    }
    Err(DatabaseError::QueryFailed) => {
        eprintln!("Query execution failed");
    }
    Err(e) => {
        eprintln!("Unexpected error: {:?}", e);
    }
}
```

## Testing

Lume includes comprehensive unit tests:

```bash
cargo test
```

Run specific test categories:

```bash
cargo test schema    # Schema definition tests
cargo test row       # Row manipulation tests
cargo test value     # Value conversion tests
cargo test registry  # Table registry tests
```

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests for any improvements.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes and improvements.
