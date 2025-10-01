#![warn(missing_docs)]

//! # Lume
//!
//! A type-safe, ergonomic schema builder and ORM for SQL databases, inspired by Drizzle ORM.
//!
//! ## Features
//!
//! - 🚀 **Type-safe**: Compile-time type checking for all database operations
//! - 🎯 **Ergonomic**: Clean, intuitive API inspired by modern ORMs
//! - ⚡ **Performance**: Zero-cost abstractions with minimal runtime overhead
//! - 🔧 **Flexible**: Support for various column constraints and SQL types
//! - 🛡️ **Safe**: Prevents SQL injection and runtime type errors
//! - 📦 **Lightweight**: Minimal dependencies, maximum functionality
//!
//! ## Quick Start
//!
//! ```no_run,ignore
//! use lume::define_schema;
//! use lume::schema::{Schema, ColumnInfo, Value};
//! use lume::database::Database;
//!
//! // Define your database schema
//! define_schema! {
//!     Users {
//!         id: i32 [primary_key().not_null()],
//!         username: String [not_null()],
//!         email: String,
//!         age: i32,
//!         is_active: bool [default_value(true)],
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to your MySQL database
//!     let db = Database::connect("mysql://user:password@localhost/database").await?;
//!     
//!     // Type-safe queries
//!     let users = db
//!         .query::<Users, QueryUsers>()
//!         .filter(lume::filter::Filter::eq_value("username", Value::String("john_doe".to_string())))
//!         .execute()
//!         .await?;
//!     
//!     for user in users {
//!         let username: Option<String> = user.get(Users::username());
//!         println!("User: {}", username.unwrap_or_default());
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Supported Database Types
//!
//! - `String` → `VARCHAR(255)`
//! - `i32` → `INTEGER`
//! - `i64` → `BIGINT`
//! - `f32` → `FLOAT`
//! - `f64` → `DOUBLE`
//! - `bool` → `BOOLEAN`
//!
//! ## Column Constraints
//!
//! - `primary_key()` - Sets the column as primary key
//! - `not_null()` - Makes the column NOT NULL
//! - `unique()` - Adds a UNIQUE constraint
//! - `indexed()` - Creates an index on the column
//! - `default_value(value)` - Sets a default value

/// Database connection and management functionality
pub mod database;

/// Query filtering and condition building
pub mod filter;

/// Database operations (queries, inserts, etc.)
pub mod operations;

/// Row abstraction for type-safe data access
pub mod row;

/// Schema definition and column management
pub mod schema;

/// Table registry and definition management
pub mod table;

mod tests;

pub(crate) enum StartingSql {
    Select,
    Insert,
    Delete,
    Update,
}

pub(crate) fn get_starting_sql(starting_sql: StartingSql, table_name: &str) -> String {
    match starting_sql {
        StartingSql::Select => "SELECT ".to_string(),
        StartingSql::Insert => format!("INSERT INTO `{}` (", table_name),
        StartingSql::Delete => format!("DELETE FROM `{}` ", table_name),
        StartingSql::Update => format!("UPDATE `{}` SET ", table_name),
    }
}

#[cfg(not(feature = "mysql"))]
pub(crate) fn returning_sql(mut sql: String, returning: &Vec<&'static str>) -> String {
    if returning.is_empty() {
        return sql;
    }

    sql.push_str(" RETURNING ");
    for (i, col) in returning.iter().enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        sql.push_str(col);
    }
    sql.push_str(";");
    sql
}

#[cfg(feature = "mysql")]
pub(crate) fn returning_sql(mut sql: String, returning: &Vec<&'static str>) -> String {
    if returning.is_empty() {
        return sql;
    }

    sql.push_str(&returning.join(", "));

    sql
}
