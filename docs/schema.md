# Schema Definition

The `define_schema!` macro is the core of Lume. It allows you to define your database schema in a type-safe, ergonomic way.

## Basic Schema

A schema definition consists of a table name and a list of columns:

```rust
use lume::define_schema;

define_schema! {
    Users {
        id: i32 [primary_key().not_null()],
        username: String [not_null()],
        email: String,
        age: i32,
    }
}
```

## Column Types

Lume supports various Rust types that map to SQL types:

| Rust Type              | SQL Type            | Notes                   |
| ---------------------- | ------------------- | ----------------------- |
| `String`               | `VARCHAR(255)`      | Text data               |
| `i8`                   | `TINYINT`           | 8-bit signed integer    |
| `i16`                  | `SMALLINT`          | 16-bit signed integer   |
| `i32`                  | `INT`               | 32-bit signed integer   |
| `i64`                  | `BIGINT`            | 64-bit signed integer   |
| `u8`                   | `TINYINT UNSIGNED`  | 8-bit unsigned integer  |
| `u16`                  | `SMALLINT UNSIGNED` | 16-bit unsigned integer |
| `u32`                  | `INT UNSIGNED`      | 32-bit unsigned integer |
| `u64`                  | `BIGINT UNSIGNED`   | 64-bit unsigned integer |
| `f32`                  | `FLOAT`             | 32-bit floating point   |
| `f64`                  | `DOUBLE`            | 64-bit floating point   |
| `bool`                 | `BOOLEAN`           | Boolean value           |
| `time::OffsetDateTime` | `DATETIME`          | Date and time           |

## Column Constraints

### Primary Key

Mark a column as the primary key:

```rust
define_schema! {
    Users {
        id: i32 [primary_key()],
        // ...
    }
}
```

### Not Null

Require a column to have a value:

```rust
define_schema! {
    Users {
        username: String [not_null()],
        // ...
    }
}
```

### Unique

Ensure column values are unique:

```rust
define_schema! {
    Users {
        email: String [unique()],
        // ...
    }
}
```

### Indexed

Create an index on a column:

```rust
define_schema! {
    Users {
        username: String [indexed()],
        // ...
    }
}
```

### Auto Increment

Automatically increment the value (for integer primary keys):

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null().auto_increment()],
        // ...
    }
}
```

### Default Values

Set a default value for a column:

```rust
define_schema! {
    Users {
        is_active: bool [default_value(true)],
        created_at: time::OffsetDateTime [default_now()],
        status: String [default_value("pending")],
    }
}
```

### Multiple Constraints

Combine multiple constraints:

```rust
define_schema! {
    Users {
        id: i32 [primary_key().not_null().auto_increment()],
        email: String [not_null().unique()],
        username: String [not_null().indexed()],
    }
}
```

## Generated Types

The `define_schema!` macro generates several types:

### Schema Struct

The main struct representing a row:

```rust
Users {
    id: i32,
    username: String,
    email: String,
    age: i32,
    is_active: Option<bool>,  // Only columns with default_value() or auto_increment() are Option<T>
}
```

### Select Struct

Used for query selection:

```rust
SelectUsers {
    id: bool,
    username: bool,
    email: bool,
    // ...
}
```

### Update Struct

Used for update operations:

```rust
UpdateUsers {
    username: Option<String>,
    email: Option<String>,
    age: Option<i32>,
    // ...
}
```

## Multiple Tables

Define multiple tables in a single macro call:

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
        content: String,
    }
}
```

## Table Registration

After defining your schema, register it with the database:

```rust
db.register_table::<Users>().await?;
db.register_table::<Posts>().await?;
```

This creates the tables in the database if they don't exist.

## Advanced Features

For more advanced schema features like:

- Custom enum types
- Generated columns
- Check constraints
- Comments and character sets

See the [Advanced Topics](advanced.md) guide.
