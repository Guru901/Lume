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
use lume::{database::Database, define_schema, filter::eq_value, schema::CustomSqlType};

// Use an enum for one of your fields!
#[derive(Clone, Debug, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Banned,
}

// If you want to support direct mapping to the database, you might want to
// implement TryFrom<String> and ToString (or use serde), but for this example let's use it as-is.

// Define your database schema, with an enum field
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        status: UserStatus [default_value(UserStatus::Active)],
        email: String,
        age: i32,
        created_at: i64 [not_null()],
    }

    Posts {
        id: i32 [primary_key().not_null()],
        title: String [not_null()],
        content: String,
        created_at: i64 [not_null()],
    }
}

impl ToString for UserStatus {
    fn to_string(&self) -> String {
        match self {
            UserStatus::Active => String::from("active"),
            UserStatus::Inactive => String::from("inactive"),
            UserStatus::Banned => String::from("banned"),
        }
    }
}

impl CustomSqlType for UserStatus {}

impl TryFrom<Value> for UserStatus {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => match s.as_str() {
                "active" => Ok(UserStatus::Active),
                "inactive" => Ok(UserStatus::Inactive),
                "banned" => Ok(UserStatus::Banned),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to your MySQL database
    let db = Database::connect("sqlite://file.db").await?;

    // Create tables (if they don't exist)
    db.register_table::<Users>().await?;
    db.register_table::<Posts>().await?;

    // Type-safe queries
    let users = db
        .query::<Users, SelectUsers>()
        .select(SelectUsers::selected().age().username().status())
        .filter(eq_value(Users::status(), UserStatus::Active.to_string()))
        .execute()
        .await?;

    for user in users {
        let username: Option<String> = user.get(Users::username());
        let age: Option<i32> = user.get(Users::age());
        let status = user.get(Users::status());
        println!(
            "User: {} (age: {}, status: {:?})",
            username.unwrap_or_default(),
            age.unwrap_or(0),
            status.unwrap_or(UserStatus::Inactive),
        );
    }

    db.insert(Users {
        id: 0,
        age: 25,
        username: "john_doe".to_string(),
        status: None,
        email: "john.doe@example.com".to_string(),
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
        let status: Option<UserStatus> = user.get(Users::status());
        println!(
            "Adult user: {}, status: {:?}",
            username.unwrap_or_default(),
            status.unwrap_or(UserStatus::Inactive)
        );
    }

    Ok(())
}
```

Read the docs for more detail.

## Contributing

We are not accepting contributions at this time. We will be accepting pull requests in the future, for now you can open an issue to discuss your ideas.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes and improvements.
