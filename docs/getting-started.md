# Getting Started

## Install

Add to Cargo.toml:

```toml
[dependencies]
lume = "0.8"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Define a Schema

```rust
use lume::define_schema;

define_schema! {
    Users {
        id: u64 [primary_key().not_null().auto_increment()],
        username: String [not_null()],
        email: String,
        is_active: bool [default_value(true)],
    }
}
```

Note: With `auto_increment()` or `default_value(...)`, fields become `Option<T>` in the generated struct.

## Connect and Create Tables

```rust
use lume::{database::Database, schema::Schema};

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let db = Database::connect("mysql://user:pass@localhost/db").await?;
db.register_table::<Users>().await?;
# Ok(())
# }
```

## Insert, Query, Update

```rust
// Insert: omit defaults/auto by passing None
let _ = db.insert(Users { id: None, username: "guru".into(), email: None, is_active: None })
    .execute()
    .await?;

// Query
let rows = db
    .query::<Users, SelectUsers>()
    .select(SelectUsers::default())
    .execute()
    .await?;

// Update
let _ = db
    .update(UpdateUsers { username: Some("new".into()), ..Default::default() })
    .filter(lume::filter::eq_value(Users::id(), 1u64))
    .execute()
    .await?;
```
