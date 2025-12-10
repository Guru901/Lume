# Getting Started

This guide will help you get started with Lume. You'll learn how to install Lume, define your first schema, and execute your first query.

## Installation

Add Lume to your `Cargo.toml` with the appropriate database feature:

```toml
[dependencies]
lume = { version = "0.12", features = ["mysql"] }
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### Database Features

Choose the database you want to use:

- **MySQL**: `features = ["mysql"]`
- **PostgreSQL**: `features = ["postgres"]`
- **SQLite**: `features = ["sqlite"]`

You can enable multiple features if you need to support multiple databases.

## Your First Schema

Let's define a simple schema for a `Users` table:

```rust
use lume::define_schema;

define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        email: String,
        age: i32,
        created_at: i64 [not_null()],
    }
}
```

This macro generates:

- A `Users` struct representing a row
- A `SelectUsers` struct for query selection
- An `UpdateUsers` struct for updates
- Column accessor functions like `Users::id()`, `Users::username()`, etc.

## Connecting to a Database

Connect to your database using a connection string:

```rust
use lume::database::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MySQL
    let db = Database::connect("mysql://user:password@localhost/database").await?;

    // PostgreSQL
    // let db = Database::connect("postgres://user:password@localhost/database").await?;

    // SQLite
    // let db = Database::connect("sqlite://file.db").await?;

    Ok(())
}
```

## Creating Tables

Register your schema to create the table in the database:

```rust
db.register_table::<Users>().await?;
```

This will create the table if it doesn't exist. The SQL is automatically generated from your schema definition.

## Inserting Data

Insert a new record:

```rust
db.insert(Users {
    id: 1,
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
    age: 30,
    created_at: 1677721600,
})
.execute()
.await?;
```

## Querying Data

Query all users:

```rust
use lume::schema::Schema;

let users = db
    .query::<Users, SelectUsers>()
    .execute()
    .await?;

for user in users {
    let username: Option<String> = user.get(Users::username());
    println!("User: {}", username.unwrap_or_default());
}
```

## Filtering Results

Filter queries with conditions:

```rust
use lume::filter::eq_value;

let users = db
    .query::<Users, SelectUsers>()
    .filter(eq_value(Users::username(), "alice"))
    .execute()
    .await?;
```

## Complete Example

Here's a complete example putting it all together:

```rust
use lume::{database::Database, define_schema, filter::eq_value};

define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        email: String,
        age: i32,
        created_at: i64 [not_null()],
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect
    let db = Database::connect("mysql://user:password@localhost/database").await?;

    // Create table
    db.register_table::<Users>().await?;

    // Insert
    db.insert(Users {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        created_at: 1677721600,
    })
    .execute()
    .await?;

    // Query
    let users = db
        .query::<Users, SelectUsers>()
        .filter(eq_value(Users::username(), "alice"))
        .execute()
        .await?;

    for user in users {
        let username: Option<String> = user.get(Users::username());
        let age: Option<i32> = user.get(Users::age());
        println!("User: {} (age: {})",
            username.unwrap_or_default(),
            age.unwrap_or(0)
        );
    }

    Ok(())
}
```

## Next Steps

- Learn about [Schema Definition](schema.md) for more advanced schema features
- Explore [Queries](queries.md) for complex query building
- Check out [Filters](filters.md) for filtering options
