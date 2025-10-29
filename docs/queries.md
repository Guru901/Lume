# Queries: Selecting and Filtering

## Selecting Columns

```rust
let rows = db
    .query::<Users, SelectUsers>()
    .select(SelectUsers::selected().username())
    .execute()
    .await?;
```

## Filters

Import helpers from `lume::filter`:

- Equality/inequality: `eq_value`, `ne_value`, `eq_column`, `ne_column`
- Null checks: `is_null`, `is_not_null`
- Comparison: `lt`, `lte`, `gt`, `gte`
- Like/ILike: `like`, `ilike`
- Between: `between`
- Arrays: `in_values`, `not_in_values`
- Boolean logic: `and`, `or`, `not_`

```rust
use lume::filter::{and, or, eq_value, gt, in_values};

let rows = db
    .query::<Users, SelectUsers>()
    .filter(or(
        and(eq_value(Users::is_active(), true), gt(Users::id(), 10u64)),
        in_values(Users::username(), vec!["alice", "bob"]),
    ))
    .limit(50)
    .offset(0)
    .execute()
    .await?;
```

## Getting Values

```rust
for row in rows {
    let id: Option<u64> = row.get(Users::id());
    let username: Option<String> = row.get(Users::username());
}
```
