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
//! ```rust
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::Row;
    use crate::schema::Value;
    use crate::schema::{ColumnInfo, Schema};
    use crate::table::TableDefinition;

    // Test schema definition
    define_schema! {
        TestUser {
            id: i32 [primary_key().not_null()],
            username: String [not_null()],
            email: String,
            age: i32,
            is_active: bool [not_null()],
        }
    }

    #[test]
    fn test_schema_definition() {
        TestUser::ensure_registered();

        // Test table name
        assert_eq!(TestUser::table_name(), "TestUser");

        // Test column retrieval
        let id_col = TestUser::id();
        let username_col = TestUser::username();
        let email_col = TestUser::email();
        let age_col = TestUser::age();
        let is_active_col = TestUser::is_active();

        assert_eq!(id_col.name(), "id");
        assert_eq!(username_col.name(), "username");
        assert_eq!(email_col.name(), "email");
        assert_eq!(age_col.name(), "age");
        assert_eq!(is_active_col.name(), "is_active");

        // Test column properties
        assert!(id_col.is_primary_key());
        assert!(!id_col.is_nullable());
        assert!(!username_col.is_nullable()); // username has not_null()
        assert!(!username_col.is_primary_key());
        assert!(!is_active_col.is_nullable()); // is_active has not_null()
    }

    #[test]
    fn test_schema_columns_info() {
        TestUser::ensure_registered();

        let columns = TestUser::get_all_columns();
        assert_eq!(columns.len(), 5);

        // Check column info
        let id_info = columns.iter().find(|c| c.name == "id").unwrap();
        assert!(id_info.primary_key);
        assert!(!id_info.nullable);
        assert_eq!(id_info.data_type, "INTEGER");

        let username_info = columns.iter().find(|c| c.name == "username").unwrap();
        assert_eq!(username_info.data_type, "VARCHAR(255)");
        assert!(!username_info.primary_key);
        assert!(!username_info.nullable); // username has not_null()

        let is_active_info = columns.iter().find(|c| c.name == "is_active").unwrap();
        assert_eq!(is_active_info.data_type, "BOOLEAN");
        assert!(!is_active_info.primary_key);
        assert!(!is_active_info.nullable); // is_active has not_null()
    }

    #[test]
    fn test_row_creation_and_manipulation() {
        let mut row = Row::<TestUser>::new();

        // Test inserting values
        let id_col = TestUser::id();
        let username_col = TestUser::username();
        let email_col = TestUser::email();
        let age_col = TestUser::age();
        let is_active_col = TestUser::is_active();

        row.insert(
            ColumnInfo {
                name: "id",
                data_type: "INTEGER",
                nullable: false,
                unique: false,
                primary_key: true,
                indexed: false,
                has_default: false,
                default_sql: None,
            },
            42,
        );

        row.insert(
            ColumnInfo {
                name: "username",
                data_type: "VARCHAR(255)",
                nullable: false,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
            },
            "testuser".to_string(),
        );

        row.insert(
            ColumnInfo {
                name: "email",
                data_type: "VARCHAR(255)",
                nullable: true,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
            },
            "test@example.com".to_string(),
        );

        row.insert(
            ColumnInfo {
                name: "age",
                data_type: "INTEGER",
                nullable: true,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
            },
            25,
        );

        row.insert(
            ColumnInfo {
                name: "is_active",
                data_type: "BOOLEAN",
                nullable: false,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
            },
            true,
        );

        // Test retrieving values
        assert_eq!(row.get(id_col), Some(42));
        assert_eq!(row.get(username_col), Some("testuser".to_string()));
        assert_eq!(row.get(email_col), Some("test@example.com".to_string()));
        assert_eq!(row.get(age_col), Some(25));
        assert_eq!(row.get(is_active_col), Some(true));
    }

    #[test]
    fn test_value_conversions() {
        // Test From implementations
        let string_val: Value = "hello".to_string().into();
        let string_val_clone = string_val.clone();
        match string_val {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String variant"),
        }

        let int_val: Value = 42.into();
        let int_val_clone = int_val.clone();
        match int_val {
            Value::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int variant"),
        }

        let long_val: Value = 1234567890123456789i64.into();
        let long_val_clone = long_val.clone();
        match long_val {
            Value::Long(l) => assert_eq!(l, 1234567890123456789i64),
            _ => panic!("Expected Long variant"),
        }

        let float_val: Value = 3.14.into();
        match float_val {
            Value::Float(f) => assert_eq!(f, 3.14),
            _ => panic!("Expected Float variant"),
        }

        let bool_val: Value = true.into();
        let bool_val_clone = bool_val.clone();
        match bool_val {
            Value::Bool(b) => assert_eq!(b, true),
            _ => panic!("Expected Bool variant"),
        }

        // Test TryFrom implementations
        let string_from_val: Result<String, ()> = string_val_clone.try_into();
        assert_eq!(string_from_val, Ok("hello".to_string()));

        let int_from_val: Result<i32, ()> = int_val_clone.clone().try_into();
        assert_eq!(int_from_val, Ok(42));

        let long_from_val: Result<i64, ()> = long_val_clone.try_into();
        assert_eq!(long_from_val, Ok(1234567890123456789i64));

        let bool_from_val: Result<bool, ()> = bool_val_clone.try_into();
        assert_eq!(bool_from_val, Ok(true));

        // Test cross-type conversion (i32 to i64)
        let long_from_int: Result<i64, ()> = int_val_clone.try_into();
        assert_eq!(long_from_int, Ok(42i64));
    }

    #[test]
    fn test_column_defaults() {
        define_schema! {
            TestDefaults {
                id: i32 [primary_key().not_null()],
                name: String [not_null()],
                score: i32 [default_value(100)],
                active: bool [default_value(true)],
            }
        }

        TestDefaults::ensure_registered();

        let score_col = TestDefaults::score();
        let active_col = TestDefaults::active();

        assert_eq!(score_col.get_default(), Some(&100));
        assert_eq!(active_col.get_default(), Some(&true));

        let columns = TestDefaults::get_all_columns();
        let score_info = columns.iter().find(|c| c.name == "score").unwrap();
        let active_info = columns.iter().find(|c| c.name == "active").unwrap();

        assert!(score_info.has_default);
        assert!(active_info.has_default);
        assert_eq!(score_info.default_sql, Some("100".to_string()));
        assert_eq!(active_info.default_sql, Some("TRUE".to_string()));
    }

    #[test]
    fn test_table_registry_idempotency() {
        // Test that registering the same table multiple times doesn't create duplicates
        TestUser::ensure_registered();
        TestUser::ensure_registered(); // Call again

        let tables = crate::table::get_all_tables();
        let user_tables: Vec<_> = tables
            .iter()
            .filter(|t| t.table_name() == "TestUser")
            .collect();

        assert_eq!(user_tables.len(), 1);
    }

    #[test]
    fn test_type_to_sql_string() {
        use crate::schema::type_to_sql_string;

        assert_eq!(type_to_sql_string::<String>(), "VARCHAR(255)");
        assert_eq!(type_to_sql_string::<i32>(), "INTEGER");
        assert_eq!(type_to_sql_string::<i64>(), "BIGINT");
        assert_eq!(type_to_sql_string::<f32>(), "FLOAT");
        assert_eq!(type_to_sql_string::<f64>(), "DOUBLE");
        assert_eq!(type_to_sql_string::<bool>(), "BOOLEAN");
    }

    #[test]
    fn test_schema_wrapper() {
        TestUser::ensure_registered();

        let wrapper = crate::schema::SchemaWrapper::<TestUser>::new();
        assert_eq!(wrapper.table_name(), "TestUser");

        let columns = wrapper.get_columns();
        assert_eq!(columns.len(), 5);

        let create_sql = wrapper.to_create_sql();
        assert!(create_sql.contains("CREATE TABLE IF NOT EXISTS TestUser"));
        assert!(create_sql.contains("id INTEGER PRIMARY KEY"));
        assert!(create_sql.contains("username VARCHAR(255) NOT NULL"));
        assert!(create_sql.contains("email VARCHAR(255)"));
        assert!(create_sql.contains("age INTEGER"));
        assert!(create_sql.contains("is_active BOOLEAN NOT NULL"));
    }
}
