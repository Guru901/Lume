# Cookbook

## Create Tables at Startup

```rust
use lume::{database::Database, schema::Schema};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    Database::connect("mysql://...").await?
        .register_table::<Users>().await?;

    Ok(())
}
```

## Paginate Results

```rust
let page = 2u64; let size = 20u64;
let rows = db
    .query::<Users, SelectUsers>()
    .limit(size as i64)
    .offset(((page - 1) * size) as i64)
    .execute()
    .await?;
```

## Partially Update a Row

```rust
use lume::filter::eq_value;
let _ = db
    .update(UpdateUsers { email: Some("new@x.com".into()), ..Default::default() })
    .filter(eq_value(Users::id(), 42u64))
    .execute()
    .await?;
```

## Raw SQL Escape Hatch

```rust
let rows = db.sql::<Users>("SELECT * FROM Users WHERE is_active = 1").await?;
```
