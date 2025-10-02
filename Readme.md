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
- 🛡️ **Safe**: Parameterized queries by default to prevent SQL injection
- 📦 **Lightweight**: Minimal dependencies, maximum functionality

## Quick Start

Add Lume to your `Cargo.toml`:

```toml
[dependencies]
lume = "0.4"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### Basic Usage

```rust
use lume::{
    database::Database,
    define_schema,
    filter::eq_value,
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

    Posts {
        id: i32 [primary_key().not_null()],
        title: String [not_null()],
        content: String,
        created_at: i64 [not_null()],
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to your MySQL database
    let db = Database::connect("mysql://user:password@localhost/database").await?;

    // Create tables (if they don't exist)
    db.register_table::<Users>().await?;
    db.register_table::<Posts>().await?;

    // Type-safe queries
    let users = db
        .query::<Users, SelectUsers>()
        .select(SelectUsers::selected().age().username())
        .filter(eq_value(Users::username(), "john_doe"))
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

    // Raw SQL (bypasses type-safe query builder; ensure your SQL matches the schema)
    let adult_users = db
        .sql::<Users>("SELECT * FROM Users WHERE age > 18")
        .await
        .unwrap();

    for user in adult_users {
        let username: Option<String> = user.get(Users::username());
        println!("Adult user: {}", username.unwrap_or_default());
    }

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
use lume::filter::{eq_value, lt};

// Filter by equality
let active_users = db
    .query::<Users, SelectUsers>()
    .filter(eq_value(Users::is_active(), true))
    .execute()
    .await?;

// Multiple filters
let young_active_users = db
    .query::<Users, SelectUsers>()
    .filter(eq_value(Users::is_active(), true))
    .filter(lt(Users::age(), 25))
    .execute()
    .await?;

// Access row data type-safely
for user in young_active_users {
    let id: Option<i32> = user.get(Users::id());
    let username: Option<String> = user.get(Users::username());
    let email: Option<String> = user.get(Users::email());

    println!(
        "User {}: {} ({})",
        id.unwrap_or(0),
        username.unwrap_or_default(),
        email.unwrap_or_default()
    );
}
```

### Joins

```rust
use lume::filter::eq_column;

// Example joining two tables using a LEFT JOIN
// Here we assume a schema with `Author` and `Book` where `Book.author_id` references `Author.id`
let authors_with_books = db
    .query::<Author, SelectAuthor>()
    .select(SelectAuthor::selected().name())
    .left_join::<Book, QueryBook>(
        eq_column(Author::id(), Book::author_id()),
        QueryBook { title: true, ..Default::default() },
    )
    .execute()
    .await?;

// INNER JOIN (returns only matching rows)
let authors_and_books = db
    .query::<Author, SelectAuthor>()
    .inner_join::<Book, QueryBook>(
        eq_column(Author::id(), Book::author_id()),
        QueryBook { title: true, ..Default::default() },
    )
    .execute()
    .await?;

// RIGHT JOIN (when supported by your database and use-case)
let right_joined = db
    .query::<Author, SelectAuthor>()
    .right_join::<Book, QueryBook>(
        eq_column(Author::id(), Book::author_id()),
        QueryBook { title: true, ..Default::default() },
    )
    .execute()
    .await?;
```

### Complex Filters

```rust
use lume::filter::{and, or, eq_value, gt_value, lt_value, in_values};

// Assuming your `Users` schema includes the referenced columns
let date_threshold = 1_696_000_000i64; // example UNIX timestamp

let users = db
    .query::<Users, SelectUsers>()
    .filter(or(
        and(
            eq_value(Users::status(), "active"),
            or(
                eq_value(Users::role(), "admin"),
                and(
                    eq_value(Users::role(), "user"),
                    gt_value(Users::created_at(), date_threshold),
                ),
            ),
        ),
        and(
            eq_value(Users::status(), "pending"),
            lt_value(Users::failed_attempts(), 3),
        ),
        in_values(Users::username(), vec!["bob", "alice", "charlie"]),
    ))
    .execute()
    .await?;
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
match db.query::<Users, SelectUsers>().execute().await {
    Ok(users) => println!("Found {} users", users.len()),
    Err(e) => eprintln!("Database error: {}", e),
}
```

### Security

- The query builder emits parameterized SQL and binds values with the driver to mitigate SQL injection.
- The `sql::<T>(...)` escape hatch accepts raw SQL; use it carefully, as you are responsible for safety and correctness.

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
