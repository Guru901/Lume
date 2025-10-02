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
//!         .query::<Users, SelectUsers>()
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

use crate::{filter::Filtered, schema::Value};

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

pub(crate) fn build_filter_expr(filter: &dyn Filtered, params: &mut Vec<Value>) -> String {
    if filter.is_or_filter() || filter.is_and_filter() {
        let op = if filter.is_or_filter() { "OR" } else { "AND" };
        let Some(f1) = filter.filter1() else {
            eprintln!("Warning: Composite filter missing filter1, using tautology");
            return "1=1".to_string();
        };
        let Some(f2) = filter.filter2() else {
            eprintln!("Warning: Composite filter missing filter2, using tautology");
            return "1=1".to_string();
        };
        let left = build_filter_expr(f1, params);
        let right = build_filter_expr(f2, params);
        return format!("({} {} {})", left, op, right);
    }

    let Some(col1) = filter.column_one() else {
        eprintln!("Warning: Simple filter missing column_one, using tautology");
        return "1=1".to_string();
    };
    // Handle IN / NOT IN array filters
    if let Some(in_array) = filter.is_in_array() {
        let values = filter.array_values().unwrap_or(&[]);
        if values.is_empty() {
            return if in_array {
                "1=0".to_string()
            } else {
                "1=1".to_string()
            };
        }
        let mut placeholders: Vec<&'static str> = Vec::with_capacity(values.len());
        for v in values.iter().cloned() {
            params.push(v);
            placeholders.push("?");
        }
        let op = if in_array { "IN" } else { "NOT IN" };
        return format!("{}.{} {} ({})", col1.0, col1.1, op, placeholders.join(", "));
    }
    if let Some(value) = filter.value() {
        match value {
            Value::Null => {
                // Special handling for NULL comparisons
                let op = filter.filter_type();
                let null_sql = match op {
                    crate::filter::FilterType::Eq => "IS NULL",
                    crate::filter::FilterType::Neq => "IS NOT NULL",
                    _ => {
                        // Unsupported operator with NULL; force false to avoid surprising results
                        return "1=0".to_string();
                    }
                };
                format!("{}.{} {}", col1.0, col1.1, null_sql)
            }
            _ => {
                params.push(value.clone());
                format!("{}.{} {} ?", col1.0, col1.1, filter.filter_type().to_sql())
            }
        }
    } else if let Some(col2) = filter.column_two() {
        format!(
            "{}.{} {} {}.{}",
            col1.0,
            col1.1,
            filter.filter_type().to_sql(),
            col2.0,
            col2.1
        )
    } else {
        return "1=1".to_string();
    }
}
