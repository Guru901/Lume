# Defaults and Auto-Increment

As of 0.8.0, Lume automatically treats columns with defaults or auto-increment as optional at the struct level and omits them in inserts when `None` is provided.

## Field Types

- `default_value(...)` → field type becomes `Option<T>`
- `auto_increment()` → field type becomes `Option<T>`

```rust
define_schema! {
    Users {
        id: u64 [primary_key().not_null().auto_increment()], // Option<u64>
        nickname: String [default_value("anon")],            // Option<String>
        username: String [not_null()],                        // String
    }
}
```

## Inserts

- If `None` is passed for a default/auto column, it is omitted from the `INSERT` statement, allowing the database to apply its default or auto-increment.

```rust
let _ = db.insert(Users {
    id: None,                // omitted → AUTO_INCREMENT
    username: "guru".into(),
    nickname: None,          // omitted → DEFAULT 'anon'
}).execute().await?;
```

## Existing Code Migration

- Some fields may now be `Option<T>` instead of `T`
- Provide `Some(value)` when you want to explicitly set a value; use `None` to defer to the database

## Why Omit Instead of Binding NULL?

Omitting the column ensures MySQL uses column defaults and AUTO_INCREMENT. Binding explicit NULL might violate NOT NULL constraints or skip default evaluation.
