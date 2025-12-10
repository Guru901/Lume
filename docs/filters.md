# Filters

Filters allow you to add WHERE conditions to your queries. Lume provides a comprehensive set of filter functions for building complex queries.

## Equality Filters

### Equal (`=`)

Match rows where a column equals a value:

```rust
use lume::filter::eq_value;

db.query::<Users, SelectUsers>()
    .filter(eq_value(Users::username(), "alice"))
    .execute()
    .await?;
```

### Not Equal (`!=`)

Match rows where a column does not equal a value:

```rust
use lume::filter::ne_value;

db.query::<Users, SelectUsers>()
    .filter(ne_value(Users::status(), "banned"))
    .execute()
    .await?;
```

## Comparison Filters

### Greater Than (`>`)

```rust
use lume::filter::gt;

db.query::<Users, SelectUsers>()
    .filter(gt(Users::age(), 18))
    .execute()
    .await?;
```

### Greater Than or Equal (`>=`)

```rust
use lume::filter::gte;

db.query::<Users, SelectUsers>()
    .filter(gte(Users::age(), 18))
    .execute()
    .await?;
```

### Less Than (`<`)

```rust
use lume::filter::lt;

db.query::<Users, SelectUsers>()
    .filter(lt(Users::age(), 65))
    .execute()
    .await?;
```

### Less Than or Equal (`<=`)

```rust
use lume::filter::lte;

db.query::<Users, SelectUsers>()
    .filter(lte(Users::age(), 29))
    .execute()
    .await?;
```

## Null Checks

### IS NULL

Match rows where a column is NULL:

```rust
use lume::filter::is_null;

db.query::<Users, SelectUsers>()
    .filter(is_null(Users::deleted_at()))
    .execute()
    .await?;
```

### IS NOT NULL

Match rows where a column is not NULL:

```rust
use lume::filter::is_not_null;

db.query::<Users, SelectUsers>()
    .filter(is_not_null(Users::email()))
    .execute()
    .await?;
```

## Array Filters

### IN

Match rows where a column value is in a list:

```rust
use lume::filter::in_array;
use lume::schema::Value;

let ids = vec![
    Value::Int32(1),
    Value::Int32(2),
    Value::Int32(3),
];

db.query::<Users, SelectUsers>()
    .filter(in_array(Users::id(), ids))
    .execute()
    .await?;
```

### NOT IN

Match rows where a column value is not in a list:

```rust
use lume::filter::not_in_array;
use lume::schema::Value;

let banned_ids = vec![
    Value::Int32(10),
    Value::Int32(20),
];

db.query::<Users, SelectUsers>()
    .filter(not_in_array(Users::id(), banned_ids))
    .execute()
    .await?;
```

## Pattern Matching

### LIKE

Match rows using a pattern (case-sensitive):

```rust
use lume::filter::like;

// Match usernames containing "doe"
db.query::<Users, SelectUsers>()
    .filter(like(Users::username(), "%doe%"))
    .execute()
    .await?;
```

### ILIKE (PostgreSQL only)

Case-insensitive pattern matching:

```rust
use lume::filter::ilike;

db.query::<Users, SelectUsers>()
    .filter(ilike(Users::username(), "%doe%"))
    .execute()
    .await?;
```

## Range Filters

### BETWEEN

Match rows where a column value is between two values (inclusive):

```rust
use lume::filter::between;

db.query::<Users, SelectUsers>()
    .filter(between(Users::age(), 18, 65))
    .execute()
    .await?;
```

## Logical Operators

### AND

Combine filters with logical AND:

```rust
use lume::filter::{and, eq_value, gt};

db.query::<Users, SelectUsers>()
    .filter(
        and(
            eq_value(Users::status(), "active"),
            gt(Users::age(), 18)
        )
    )
    .execute()
    .await?;
```

### OR

Combine filters with logical OR:

```rust
use lume::filter::{or, eq_value};

db.query::<Users, SelectUsers>()
    .filter(
        or(
            eq_value(Users::status(), "active"),
            eq_value(Users::status(), "pending")
        )
    )
    .execute()
    .await?;
```

### NOT

Negate a filter:

```rust
use lume::filter::{not, eq_value};

db.query::<Users, SelectUsers>()
    .filter(not(eq_value(Users::status(), "banned")))
    .execute()
    .await?;
```

## Complex Filter Combinations

Combine multiple logical operators:

```rust
use lume::filter::{and, or, eq_value, gt, lt};

db.query::<Users, SelectUsers>()
    .filter(
        and(
            gt(Users::age(), 18),
            or(
                eq_value(Users::status(), "active"),
                eq_value(Users::status(), "pending")
            ),
            not(eq_value(Users::banned(), true))
        )
    )
    .execute()
    .await?;
```

## Column-to-Column Comparisons

Compare two columns:

```rust
use lume::filter::eq_column;

// Find users where created_at equals updated_at
db.query::<Users, SelectUsers>()
    .filter(eq_column(Users::created_at(), Users::updated_at()))
    .execute()
    .await?;
```

## Multiple Filters

You can chain multiple `.filter()` calls (they are combined with AND):

```rust
db.query::<Users, SelectUsers>()
    .filter(eq_value(Users::status(), "active"))
    .filter(gt(Users::age(), 18))
    .filter(is_not_null(Users::email()))
    .execute()
    .await?;
```

This is equivalent to:

```rust
db.query::<Users, SelectUsers>()
    .filter(
        and(
            and(
                eq_value(Users::status(), "active"),
                gt(Users::age(), 18)
            ),
            is_not_null(Users::email())
        )
    )
    .execute()
    .await?;
```

## Type Safety

All filters are type-checked at compile time:

```rust
// ✅ Works - username is String
eq_value(Users::username(), "alice")

// ✅ Works - age is i32
gt(Users::age(), 18)

// ❌ Compile error - wrong type
// eq_value(Users::username(), 123)
```

## Best Practices

1. **Use indexes** - Mark frequently filtered columns with `indexed()` in your schema
2. **Filter early** - Apply filters before joins when possible
3. **Use appropriate operators** - Use `IN` for multiple values instead of multiple `OR` conditions
4. **Combine logically** - Use `and()` and `or()` for complex conditions

## Next Steps

- Learn about [Queries](queries.md) for query building
- Check out [Joins](joins.md) for joining tables with filters
- See [Advanced Topics](advanced.md) for more advanced patterns
