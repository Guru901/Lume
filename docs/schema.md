# Defining Schemas

Use `define_schema!` to declare tables and columns with types and constraints.

## Types → SQL

- String → VARCHAR(255)
- i8/i16/i32/i64 → TINYINT/SMALLINT/INTEGER/BIGINT
- u8/u16/u32/u64 → UNSIGNED variants
- f32/f64 → FLOAT/DOUBLE
- bool → BOOLEAN
- Others → TEXT

## Constraints

- primary_key(), not_null(), unique(), indexed()
- default_value(value)
- auto_increment() [MySQL]
- comment(text), charset(cs), collate(col)
- on_update_current_timestamp()
- invisible(), check(expr), generated_virtual(expr), generated_stored(expr)

## Field Type Inference

If a column has `default_value(...)` or `auto_increment()`, the generated struct field becomes `Option<T>`. Passing `None` lets the database supply the value.

```rust
define_schema! {
    Users {
        id: u64 [primary_key().not_null().auto_increment()], // Option<u64>
        username: String [not_null()],                        // String
        nickname: String [default_value("anon")],            // Option<String>
    }
}
```

## Select and Update Structs

The macro generates:

- `SelectUsers` with boolean flags for each column
- `UpdateUsers` with `Option<T>` fields; only `Some` values are updated

```rust
let selection = SelectUsers::selected().username(); // username=true
let update = UpdateUsers { nickname: Some("pro".into()), ..Default::default() };
```
