# Inserts and Updates

## Inserts

- Build an `INSERT` from a struct instance
- Columns with `None` and having a default or auto-increment are omitted so the DB fills them

```rust
// Users { id: Option<u64>, username: String, email: Option<String>, is_active: Option<bool> }
let _ = db.insert(Users {
    id: None,
    username: "guru".into(),
    email: None,
    is_active: None,
})
.returning(SelectUsers::selected())
.execute()
.await?;
```

## Updates

- Use `UpdateUsers` with `Option<T>` fields
- Only `Some` fields are included in the `SET` clause

```rust
use lume::filter::eq_value;

let _ = db
    .update(UpdateUsers { email: Some("new@example.com".into()), ..Default::default() })
    .filter(eq_value(Users::id(), 1u64))
    .execute()
    .await?;
```

## Returning

- `Insert` and `InsertMany` support `returning(...)` to fetch inserted rows (by id via last_insert_id)

```rust
let rows = db
    .insert(Users { id: None, username: "x".into(), email: None, is_active: None })
    .returning(SelectUsers::selected())
    .execute()
    .await?;
```
