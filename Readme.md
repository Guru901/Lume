# Lume

An Erganmoic and developer-friendly Schema Builder for SQL Databases.

[![Crates.io](https://img.shields.io/crates/v/lume)](https://crates.io/crates/lume)
[![Documentation](https://img.shields.io/docsrs/lume)](https://docs.rs/lume)
[![License](https://img.shields.io/crates/l/lume)](LICENSE)

```rust
use lume::database::{DatabaseError, connect};
use lume::define_columns;
use lume::schema::Column;
use lume::schema::Schema;

// Erganomic schema definition
define_columns! {
    Users {
        username: String,
        id: i32 [primary_key().not_null()]
    }

    Posts {
        id: i32 [primary_key()],
        title: String [not_null()],
        content: String,
    }
}

#[tokio::main]
async fn main() -> Result<(), DatabaseError> {
    let db = connect("postgresql://...");

    // Typesafe query
    let rows = db
        .query::<Users>()
        .filter(Users::username(), "guru".to_string())
        .await?;

    for row in rows {
        let username: Option<String> = row.get(Users::username());
        println!("User: {:?}", username);
    }

    Ok(())
}
```
