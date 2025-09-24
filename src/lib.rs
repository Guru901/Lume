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
//!         .query::<Users>()
//!         .filter(lume::filter::Filter::eq("username", Value::String("john_doe".to_string())))
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
