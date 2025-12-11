# Lume

A type-safe, ergonomic query builder and ORM for SQL databases, inspired by Drizzle ORM.

[![Crates.io](https://img.shields.io/crates/v/lume)](https://crates.io/crates/lume)
[![Documentation](https://img.shields.io/docsrs/lume)](https://docs.rs/lume)
[![License](https://img.shields.io/crates/l/lume)](LICENSE)

## Features

- 🚀 **Type-safe**: Compile-time type checking for all database operations
- 🎯 **Ergonomic**: Clean, intuitive API inspired by modern ORMs
- ⚡ **Performance**: Zero-cost abstractions with minimal runtime overhead
- 🔧 **Flexible**: Support for MySQL, PostgreSQL, and SQLite
- 🛡️ **Safe**: Parameterized queries by default to prevent SQL injection
- 📦 **Lightweight**: Minimal dependencies, maximum functionality

## Quick Start

Add Lume to your `Cargo.toml`:

```toml
[dependencies]
lume = { version = "0.12", features = ["mysql"] }
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### Basic Example

```rust
use lume::{database::Database, define_schema, filter::eq_value};

// Define your database schema
define_schema! {
    Users {
        id: Uuid [primary_key().not_null().default_random()],
        username: String [not_null()],
        email: String,
        age: i32,
        created_at: i64 [not_null()],
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to your database
    let db = Database::connect("mysql://user:password@localhost/database").await?;

    // Create tables (if they don't exist)
    db.register_table::<Users>().await?;

    // Insert a new user
    db.insert(Users {
        id: None,
        username: "john_doe".to_string(),
        email: "john.doe@example.com".to_string(),
        age: 25,
        created_at: 1677721600,
    })
    .execute()
    .await?;

    // Query users
    let users = db
        .query::<Users, SelectUsers>()
        .filter(eq_value(Users::username(), "john_doe"))
        .execute()
        .await?;

    for user in users {
        let username: Option<String> = user.get(Users::username());
        println!("User: {}", username.unwrap_or_default());
    }

    Ok(())
}
```

## Documentation

- [Getting Started](docs/getting-started.md) - Installation and basic usage
- [Schema Definition](docs/schema.md) - Defining database schemas
- [Queries](docs/queries.md) - Building and executing queries
- [Filters](docs/filters.md) - Filtering and conditions
- [Inserts & Updates](docs/inserts-updates.md) - Inserting and updating data
- [Joins](docs/joins.md) - Joining tables
- [Advanced Topics](docs/advanced.md) - Advanced features and patterns

## Supported Databases

Lume supports multiple database backends through feature flags:

- **MySQL**: `lume = { version = "0.12", features = ["mysql"] }`
- **PostgreSQL**: `lume = { version = "0.12", features = ["postgres"] }`
- **SQLite**: `lume = { version = "0.12", features = ["sqlite"] }`

## Type Mapping

Lume automatically maps Rust types to SQL types:

| Rust Type              | SQL Type            |
| ---------------------- | ------------------- |
| `String`               | `VARCHAR(255)`      |
| `i8`                   | `TINYINT`           |
| `i16`                  | `SMALLINT`          |
| `i32`                  | `INT`               |
| `i64`                  | `BIGINT`            |
| `u8`                   | `TINYINT UNSIGNED`  |
| `u16`                  | `SMALLINT UNSIGNED` |
| `u32`                  | `INT UNSIGNED`      |
| `u64`                  | `BIGINT UNSIGNED`   |
| `f32`                  | `FLOAT`             |
| `f64`                  | `DOUBLE`            |
| `bool`                 | `BOOLEAN`           |
| `time::OffsetDateTime` | `DATETIME`          |

## Contributing

We are not accepting contributions at this time. We will be accepting pull requests in the future, for now you can open an issue to discuss your ideas.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes and improvements.
