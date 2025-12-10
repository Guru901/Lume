# Joins

Lume supports joining multiple tables in your queries, allowing you to fetch related data in a single query. Lume provides several join types: `left_join`, `inner_join`, `right_join`, `full_join`, and `cross_join`.

## Join Types

Lume supports the following join types:

- **LEFT JOIN** - Returns all rows from the left table and matching rows from the right table
- **INNER JOIN** - Returns only rows that have matching values in both tables
- **RIGHT JOIN** - Returns all rows from the right table and matching rows from the left table (not available for SQLite)
- **FULL JOIN** - Returns all rows from both tables (PostgreSQL only)
- **CROSS JOIN** - Produces a Cartesian product of all rows from both tables

## Basic Left Join

Join two tables using a LEFT JOIN:

```rust
use lume::filter::eq_column;

define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
    }

    Posts {
        id: i32 [primary_key().not_null()],
        user_id: i32 [not_null()],
        title: String [not_null()],
        content: String,
    }
}

// Join Posts to Users using LEFT JOIN
let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, content: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Inner Join

Use INNER JOIN to get only matching rows:

```rust
let results = db
    .query::<Users, SelectUsers>()
    .inner_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Right Join

Use RIGHT JOIN (not available for SQLite):

```rust
let results = db
    .query::<Users, SelectUsers>()
    .right_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Full Join

Use FULL JOIN (PostgreSQL only):

```rust
let results = db
    .query::<Users, SelectUsers>()
    .full_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Cross Join

Use CROSS JOIN for a Cartesian product (no join condition needed):

```rust
let results = db
    .query::<Users, SelectUsers>()
    .cross_join::<Posts, SelectPosts>(
        SelectPosts { title: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Join Condition

The join condition uses `eq_column()` to specify how tables are related:

```rust
// Users.id = Posts.user_id
eq_column(Users::id(), Posts::user_id())
```

## Selecting Columns in Joins

When joining, you must specify which columns to select from the joined table using the Select struct:

```rust
// Select specific columns from the joined table
.left_join::<Posts, SelectPosts>(
    eq_column(Users::id(), Posts::user_id()),
    SelectPosts {
        title: true,
        content: true,
        ..Default::default()
    }
)
```

Set fields to `true` for columns you want to select, and use `..Default::default()` for the rest.

## Multiple Joins

Chain multiple joins:

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
    }

    Posts {
        id: i32 [primary_key().not_null()],
        user_id: i32 [not_null()],
        title: String [not_null()],
    }

    Comments {
        id: i32 [primary_key().not_null()],
        post_id: i32 [not_null()],
        user_id: i32 [not_null()],
        content: String [not_null()],
    }
}

// Join Users -> Posts -> Comments
let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, ..Default::default() }
    )
    .inner_join::<Comments, SelectComments>(
        eq_column(Posts::id(), Comments::post_id()),
        SelectComments { content: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Joins with Filters

Add filters to joined queries:

```rust
use lume::filter::{eq_column, eq_value};

let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, ..Default::default() }
    )
    .filter(eq_value(Users::username(), "alice"))
    .filter(eq_value(Posts::title(), "My Post"))
    .execute()
    .await?;
```

## Selecting Specific Columns from Main Table

You can also select specific columns from the main table:

```rust
let results = db
    .query::<Users, SelectUsers>()
    .select(SelectUsers { username: true, ..Default::default() })
    .left_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, content: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Accessing Joined Data

After executing a joined query, access data from all joined tables:

```rust
let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, content: true, ..Default::default() }
    )
    .execute()
    .await?;

for row in results {
    // Access Users columns
    let username: Option<String> = row.get(Users::username());

    // Access Posts columns
    let title: Option<String> = row.get(Posts::title());
    let content: Option<String> = row.get(Posts::content());

    println!("User: {}, Post: {}",
        username.unwrap_or_default(),
        title.unwrap_or_default()
    );
}
```

## Complex Join Conditions

You can use more complex join conditions with filters:

```rust
use lume::filter::{and, eq_column, eq_value};

let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Posts, SelectPosts>(
        and(
            eq_column(Users::id(), Posts::user_id()),
            eq_value(Posts::status(), "published")
        ),
        SelectPosts { title: true, ..Default::default() }
    )
    .execute()
    .await?;
```

## Self Joins

Join a table to itself:

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        manager_id: i32,  // References another user
    }
}

// Find users and their managers
let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Users, SelectUsers>(
        eq_column(Users::id(), Users::manager_id()),
        SelectUsers { username: true, ..Default::default() }
    )
    .execute()
    .await?;
```

Note: Self joins may require aliasing, which may have limitations in the current implementation.

## Database-Specific Join Support

- **MySQL**: Supports LEFT, INNER, RIGHT, and CROSS joins
- **PostgreSQL**: Supports LEFT, INNER, RIGHT, FULL, and CROSS joins
- **SQLite**: Supports LEFT, INNER, and CROSS joins (no RIGHT or FULL joins)

## Performance Considerations

1. **Index foreign keys** - Ensure foreign key columns are indexed:

   ```rust
   define_schema! {
       Posts {
           user_id: i32 [not_null().indexed()],
       }
   }
   ```

2. **Select only needed columns** - Specify only the columns you need in the Select struct

3. **Filter before joining** - Apply filters early when possible:

   ```rust
   // Better: filter first
   db.query::<Users, SelectUsers>()
       .filter(eq_value(Users::status(), "active"))
       .left_join::<Posts, SelectPosts>(/* ... */)
       .execute()
       .await?;
   ```

4. **Limit results** - Use `.limit()` to prevent fetching too much data

## Common Patterns

### User with Posts

```rust
let results = db
    .query::<Users, SelectUsers>()
    .left_join::<Posts, SelectPosts>(
        eq_column(Users::id(), Posts::user_id()),
        SelectPosts { title: true, content: true, ..Default::default() }
    )
    .filter(eq_value(Users::id(), 1))
    .execute()
    .await?;
```

### Posts with Comments

```rust
let results = db
    .query::<Posts, SelectPosts>()
    .inner_join::<Comments, SelectComments>(
        eq_column(Posts::id(), Comments::post_id()),
        SelectComments { content: true, ..Default::default() }
    )
    .filter(eq_value(Posts::id(), 1))
    .execute()
    .await?;
```

## Next Steps

- Learn about [Queries](queries.md) for more query features
- Check out [Filters](filters.md) for join conditions
- See [Advanced Topics](advanced.md) for raw SQL when needed
