# Queries

Lume provides a type-safe query builder for selecting data from your database.

## Basic Query

Query all rows from a table:

```rust
use lume::schema::Schema;

let users = db
    .query::<Users, SelectUsers>()
    .execute()
    .await?;
```

## Selecting Specific Columns

By default, all columns are selected. You can specify which columns to select:

```rust
let users = db
    .query::<Users, SelectUsers>()
    .select(SelectUsers::selected().username().email())
    .execute()
    .await?;
```

## Filtering

Add WHERE conditions to your query:

```rust
use lume::filter::eq_value;

let users = db
    .query::<Users, SelectUsers>()
    .filter(eq_value(Users::username(), "alice"))
    .execute()
    .await?;
```

You can chain multiple filters (they are combined with AND):

```rust
use lume::filter::{eq_value, gt};

let users = db
    .query::<Users, SelectUsers>()
    .filter(eq_value(Users::status(), "active"))
    .filter(gt(Users::age(), 18))
    .execute()
    .await?;
```

See the [Filters](filters.md) guide for all available filter functions.

## Limiting Results

Limit the number of results:

```rust
let users = db
    .query::<Users, SelectUsers>()
    .limit(10)
    .execute()
    .await?;
```

## Pagination

Use `limit` and `offset` for pagination:

```rust
// Get page 2 (items 11-20)
let users = db
    .query::<Users, SelectUsers>()
    .limit(10)
    .offset(10)
    .execute()
    .await?;
```

## Distinct Results

Get distinct results:

```rust
let users = db
    .query::<Users, SelectUsers>()
    .select_distinct(SelectUsers::selected().username())
    .execute()
    .await?;
```

## Joins

Join multiple tables:

```rust
use lume::filter::eq_column;

let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, ..Default::default() }
    )
    .execute()
    .await?;
```

See the [Joins](joins.md) guide for more details.

## Reading Results

After executing a query, you get a vector of `Row<T>` objects. Access column values:

```rust
let users = db
    .query::<Users, SelectUsers>()
    .execute()
    .await?;

for user in users {
    let id: Option<i32> = user.get(Users::id());
    let username: Option<String> = user.get(Users::username());
    let age: Option<i32> = user.get(Users::age());

    println!("ID: {}, Username: {}, Age: {}",
        id.unwrap_or(0),
        username.unwrap_or_default(),
        age.unwrap_or(0)
    );
}
```

The `get()` method returns `Option<T>` because:

- The column might not have been selected in the query
- The column might be NULL in the database

**Note**: This is different from the schema struct field types. Schema struct fields are `T` for nullable columns (without `not_null()`), and only `Option<T>` for columns with `default_value()` or `auto_increment()`. The `get()` method always returns `Option<T>` because it's retrieving from database results.

## Complex Queries

Build complex queries by combining filters:

```rust
use lume::filter::{and, or, eq_value, gt, lt};

let users = db
    .query::<Users, SelectUsers>()
    .filter(
        and(
            gt(Users::age(), 18),
            or(
                eq_value(Users::status(), "active"),
                eq_value(Users::status(), "pending")
            )
        )
    )
    .limit(50)
    .execute()
    .await?;
```

## Query Builder Pattern

The query builder is fluent and chainable:

```rust
let query = db.query::<Users, SelectUsers>();

// Add filters
let query = query.filter(eq_value(Users::status(), "active"));

// Add limits
let query = query.limit(10).offset(0);

// Execute
let users = query.execute().await?;
```

## Type Safety

All queries are type-checked at compile time:

```rust
// ✅ This works - username is a String column
db.query::<Users, SelectUsers>()
    .filter(eq_value(Users::username(), "alice"))
    .execute()
    .await?;

// ❌ This won't compile - wrong type
// db.query::<Users, SelectUsers>()
//     .filter(eq_value(Users::username(), 123))  // Error!
//     .execute()
//     .await?;
```

## Performance Tips

1. **Select only needed columns** - Use `.select()` to limit data transfer
2. **Use indexes** - Mark frequently filtered columns with `indexed()`
3. **Use limits** - Always use `.limit()` when you don't need all results
4. **Filter early** - Apply filters before joins when possible

## Next Steps

- Learn about [Filters](filters.md) for all filtering options
- Check out [Joins](joins.md) for joining tables
- See [Advanced Topics](advanced.md) for raw SQL queries
