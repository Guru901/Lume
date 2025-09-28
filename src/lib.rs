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

use crate::{
    filter::Filter,
    operations::query::{JoinInfo, JoinType},
    schema::{Select, Value},
};

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
}

pub(crate) fn get_starting_sql(starting_sql: StartingSql) -> String {
    match starting_sql {
        StartingSql::Select => "SELECT ".to_string(),
        StartingSql::Insert => "INSERT INTO ".to_string(),
    }
}

pub(crate) fn select_sql<S: Select>(
    mut sql: String,
    select: Option<S>,
    table_name: &str,
) -> String {
    if select.is_some() {
        sql.push_str(&select.unwrap().get_selected().join(", "));
    } else {
        sql.push_str("*");
    }

    sql.push_str(format!(" FROM {}", table_name).as_str());
    sql
}

pub(crate) fn joins_sql(mut sql: String, joins: &Vec<JoinInfo>) -> String {
    if joins.is_empty() {
        return sql;
    }

    for join in joins {
        let join_type = match join.join_type {
            JoinType::Left => "LEFT JOIN",
        };

        let join_table = &join.table_name;

        sql.push_str(&format!(
            " {} {} ON {}.{} = {}.{}",
            join_type,
            join_table,
            join.condition.column_one.0,
            join.condition.column_one.1,
            join.condition.column_two.as_ref().unwrap().0,
            join.condition.column_two.as_ref().unwrap().1
        ));
    }

    sql
}

pub(crate) fn filter_sql(mut sql: String, filters: Vec<Filter>) -> String {
    if filters.is_empty() {
        return sql;
    }

    sql.push_str(" WHERE ");

    for (i, filter) in filters.iter().enumerate() {
        if let Some(value) = &filter.value {
            match value {
                Value::String(inner) => {
                    let escaped = inner.replace('\'', "''");
                    let filter_sql = format!(
                        "{}.{} {} '{}'",
                        filter.column_one.0,
                        filter.column_one.1,
                        filter.filter_type.to_sql(),
                        escaped
                    );
                    sql.push_str(&filter_sql);
                }
                _ => {
                    let filter_sql = format!(
                        "{}.{} {} {}",
                        filter.column_one.0,
                        filter.column_one.1,
                        filter.filter_type.to_sql(),
                        value
                    );
                    sql.push_str(&filter_sql);
                }
            }
        }
        if let Some(column) = &filter.column_two {
            sql.push_str(&format!(
                "{}.{} {} {}.{}",
                filter.column_one.0,
                filter.column_one.1,
                filter.filter_type.to_sql(),
                column.0,
                column.1
            ));
        }

        if i < filters.len() - 1 {
            sql.push_str(" AND ");
        }
    }

    sql
}
