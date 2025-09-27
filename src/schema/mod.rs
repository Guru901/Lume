#![warn(missing_docs)]

//! # Schema Module
//!
//! This module provides the core schema definition functionality for Lume.
//! It includes the schema trait, column definitions, and the powerful `define_schema!` macro.
//!
//! ## Key Components
//!
//! - [`Schema`] trait: Core trait for all database schemas
//! - [`Column<T>`]: Type-safe column definition with constraints
//! - [`Value`]: Enum for storing and converting between database values
//! - [`define_schema!`]: Macro for ergonomic schema definition
//!
//! ## Example
//!
//! ```no_run,ignore
//! use lume::define_schema;
//!
//! define_schema! {
//!     User {
//!         id: i32 [primary_key().not_null()],
//!         name: String [not_null()],
//!         email: String [unique()],
//!         age: i32,
//!         active: bool [default_value(true)],
//!     }
//! }
//! ```

mod column;

use std::collections::HashMap;
use std::marker::PhantomData;

use crate::table::TableDefinition;
pub use column::Column;
pub use column::Value;
pub use column::convert_to_value;

/// Core trait that all database schemas must implement.
///
/// This trait provides the interface for schema registration, column retrieval,
/// and table metadata. It's automatically implemented by the `define_schema!` macro.
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
/// use lume::schema::{Schema, ColumnInfo};
///
/// define_schema! {
///     Product {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
/// }
///
/// // The Schema trait is automatically implemented
/// assert_eq!(Product::table_name(), "Product");
/// let columns = Product::get_all_columns();
/// ```
pub trait Schema {
    /// Returns the table name for this schema.
    ///
    /// This is used for SQL generation and table registry.
    fn table_name() -> &'static str;

    /// Returns metadata for all columns in this schema.
    ///
    /// This includes column names, types, constraints, and other metadata
    /// needed for SQL generation and type checking.
    fn get_all_columns() -> Vec<ColumnInfo>;

    /// Ensures the schema is registered in the table registry.
    ///
    /// This method is idempotent and can be called multiple times safely.
    /// It's automatically called when using the generated schema methods.
    fn ensure_registered();

    /// Returns a map of column names to their corresponding values for this schema instance.
    ///
    /// This method is used to extract the values of all fields in the schema as a
    /// `HashMap<String, Value>`, where each key is the column name and each value is
    /// the associated database value. This is primarily used for insert and update
    /// operations to serialize the struct into a form suitable for database interaction.
    fn values(&self) -> HashMap<String, Value>;
}

pub trait Select {
    fn default() -> Self;

    fn get_selected(self) -> Vec<String>;
}

/// Metadata information for a database column.
///
/// This struct contains all the necessary information about a column
/// for SQL generation, type checking, and constraint validation.
///
/// # Fields
///
/// - `name`: The column name in the database
/// - `data_type`: The SQL data type (e.g., "INTEGER", "VARCHAR(255)")
/// - `nullable`: Whether the column allows NULL values
/// - `unique`: Whether the column has a UNIQUE constraint
/// - `primary_key`: Whether the column is a primary key
/// - `indexed`: Whether the column has an index
/// - `has_default`: Whether the column has a default value
/// - `default_sql`: The SQL representation of the default value
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// The column name in the database
    pub name: &'static str,
    /// The SQL data type (e.g., "INTEGER", "VARCHAR(255)")
    pub data_type: &'static str,
    /// Whether the column allows NULL values
    pub nullable: bool,
    /// Whether the column has a UNIQUE constraint
    pub unique: bool,
    /// Whether the column is a primary key
    pub primary_key: bool,
    /// Whether the column has an index
    pub indexed: bool,
    /// Whether the column has a default value
    pub has_default: bool,
    /// The SQL representation of the default value
    pub default_sql: Option<String>,
}

/// Defines a database schema with type-safe columns and constraints.
///
/// This macro creates a schema struct that implements the [`Schema`] trait
/// and provides type-safe access to column definitions.
///
/// # Syntax
///
/// ```rust
/// use lume::define_schema;
/// use lume::schema::{Schema, ColumnInfo};
///
/// define_schema! {
///     TableName {
///         column_name: i32 [primary_key()],
///         another_column: String [not_null()],
///     }
/// }
/// ```
///
/// # Column Constraints
///
/// - `primary_key()` - Sets the column as primary key
/// - `not_null()` - Makes the column NOT NULL
/// - `unique()` - Adds a UNIQUE constraint
/// - `indexed()` - Creates an index on the column
/// - `default_value(value)` - Sets a default value
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
/// use lume::schema::{Schema, ColumnInfo};
///
/// define_schema! {
///     User {
///         id: i32 [primary_key().not_null()],
///         username: String [not_null().unique()],
///         email: String [not_null()],
///         age: i32,
///         is_active: bool [default_value(true)],
///         created_at: i64 [not_null()],
///     }
/// }
///
/// // Access columns type-safely
/// let id_col = User::id();
/// let username_col = User::username();
///
/// // Get schema information
/// assert_eq!(User::table_name(), "User");
/// let columns = User::get_all_columns();
/// ```
///
/// # Generated Code
///
/// This macro generates:
/// - A struct with the given name
/// - Column accessor methods that return `&'static Column<T>`
/// - Implementation of the [`Schema`] trait
/// - Automatic table registration
#[macro_export]
macro_rules! define_schema {
    (
        $(
            $struct_name:ident {
            $(
                $name:ident: $type:ty $([ $($args:tt)* ])?
            ),* $(,)?
        }
    )*
    ) => {
             // Auto-register the table when the struct is defined
             #[allow(non_upper_case_globals)]
             static _REGISTER: std::sync::Once = std::sync::Once::new();
             use $crate::table::register_table;
             use $crate::schema::type_to_sql_string;
             use $crate::schema::DefaultToSql;
             use std::collections::HashMap;
             use $crate::schema::Value;
             use $crate::schema::convert_to_value;
             use $crate::schema::Select;
             use paste::paste;

        $(
        #[derive(Debug)]
        pub struct $struct_name {
            $(
                pub $name: $type,
            )*
        }

        paste! {
            #[derive(Debug)]
            pub struct [<Query $struct_name>] {
                $(
                    pub $name: bool,
                )*
            }

            impl Default for [<Query $struct_name>] {
                fn default() -> Self {
                    Self {
                        $(
                            $name: true,
                        )*
                    }
                }
            }

            impl Select for [<Query $struct_name>] {
                fn default() -> Self {
                    Self {
                        $(
                            $name: true,
                        )*
                    }
                }

                fn get_selected(self) -> Vec<String> {
                    let mut vec = Vec::new();

                    $(
                        if self.$name {
                            vec.push(stringify!($name).to_string())
                        }
                    )*

                    vec
                }
            }
        }


        impl $struct_name {
            $(
                pub fn $name() -> &'static $crate::schema::Column<$type> {
                    static COL: std::sync::OnceLock<$crate::schema::Column<$type>> = std::sync::OnceLock::new();
                    COL.get_or_init(|| {
                        $crate::schema::Column::<$type>::new(stringify!($name), stringify!($struct_name))
                            $(.$($args)*)?
                    })
                }
            )*
        }



        impl Schema for $struct_name {
            fn table_name() -> &'static str {
                stringify!($struct_name)
            }

            fn values(&self) -> HashMap<String, Value> {
                let mut map = HashMap::new();
                $(
                    map.insert(
                        stringify!($name).to_string(),
                        convert_to_value(&self.$name)
                    );
                )*
                map
            }
            fn ensure_registered() {
                // Function-local static to avoid name collisions across macro expansions
                static REGISTER: std::sync::Once = std::sync::Once::new();
                REGISTER.call_once(|| {
                    register_table::<$struct_name>();
                });
            }

            fn get_all_columns() -> Vec<ColumnInfo> {
                vec![
                    $(
                        {
                            let col = Self::$name();

                            ColumnInfo {
                                name: col.name(),
                                data_type: type_to_sql_string::<$type>(),
                                nullable: col.is_nullable(),
                                unique: col.is_unique(),
                                primary_key: col.is_primary_key(),
                                indexed: col.is_indexed(),
                                has_default: col.get_default().is_some(),
                                default_sql: col.default_to_sql(),
                            }
                        }
                    ),*
                ]
            }
        }
        )*
    };
}

/// Converts a Rust type to its corresponding SQL type string.
///
/// This function provides the mapping between Rust types and SQL column types
/// used in database schema generation.
///
/// # Supported Types
///
/// - `String` → `"VARCHAR(255)"`
/// - `i32` → `"INTEGER"`
/// - `i64` → `"BIGINT"`
/// - `f32` → `"FLOAT"`
/// - `f64` → `"DOUBLE"`
/// - `bool` → `"BOOLEAN"`
/// - All other types → `"TEXT"` (fallback)
///
/// # Example
///
/// ```rust
/// use lume::schema::type_to_sql_string;
///
/// assert_eq!(type_to_sql_string::<String>(), "VARCHAR(255)");
/// assert_eq!(type_to_sql_string::<i32>(), "INTEGER");
/// assert_eq!(type_to_sql_string::<i64>(), "BIGINT");
/// assert_eq!(type_to_sql_string::<bool>(), "BOOLEAN");
/// ```
pub fn type_to_sql_string<T: 'static>() -> &'static str {
    use std::any::TypeId;

    let type_id = TypeId::of::<T>();

    if type_id == TypeId::of::<String>() {
        "VARCHAR(255)"
    } else if type_id == TypeId::of::<i32>() {
        "INTEGER"
    } else if type_id == TypeId::of::<i64>() {
        "BIGINT"
    } else if type_id == TypeId::of::<f32>() {
        "FLOAT"
    } else if type_id == TypeId::of::<f64>() {
        "DOUBLE"
    } else if type_id == TypeId::of::<bool>() {
        "BOOLEAN"
    } else {
        "TEXT" // fallback
    }
}

/// A wrapper around a schema type that implements [`TableDefinition`].
///
/// This struct is used internally to bridge between the [`Schema`] trait
/// and the [`TableDefinition`] trait for table registry and SQL generation.
///
/// # Type Parameters
///
/// - `T`: The schema type that implements [`Schema`]
pub(crate) struct SchemaWrapper<T: Schema> {
    _phantom: PhantomData<T>,
}

// Implement Clone for SchemaWrapper<T>
impl<T: Schema> Clone for SchemaWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Schema> SchemaWrapper<T> {
    /// Creates a new `SchemaWrapper` instance.
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Schema + Sync + Send + 'static> TableDefinition for SchemaWrapper<T> {
    fn table_name(&self) -> &'static str {
        T::table_name()
    }

    fn get_columns(&self) -> Vec<ColumnInfo> {
        T::get_all_columns()
    }

    fn to_create_sql(&self) -> String {
        let table_name = self.table_name();
        let columns = self.get_columns();

        let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", table_name);

        let column_definitions: Vec<String> = columns
            .iter()
            .map(|col| {
                let mut def = format!("    {} {}", col.name, col.data_type);

                if col.primary_key {
                    def.push_str(" PRIMARY KEY");
                }

                if !col.nullable && !col.primary_key {
                    def.push_str(" NOT NULL");
                }

                if col.unique && !col.primary_key {
                    def.push_str(" UNIQUE");
                }

                if let Some(ref default) = col.default_sql {
                    def.push_str(&format!(" DEFAULT {}", default));
                }

                def
            })
            .collect();

        sql.push_str(&column_definitions.join(",\n"));
        sql.push_str("\n);");

        // Add indexes
        let indexes: Vec<String> = columns
            .iter()
            .filter(|col| col.indexed && !col.primary_key)
            .map(|col| {
                format!(
                    "CREATE INDEX idx_{}_{} ON {} ({});",
                    table_name, col.name, table_name, col.name
                )
            })
            .collect();

        if !indexes.is_empty() {
            sql.push_str("\n\n");
            sql.push_str(&indexes.join("\n"));
        }

        sql
    }

    fn clone_box(&self) -> Box<dyn TableDefinition> {
        Box::new(self.clone())
    }
}

/// Trait for converting column default values to SQL representation.
///
/// This trait is implemented for all supported column types to provide
/// proper SQL formatting of default values in CREATE TABLE statements.
///
/// # Example
///
/// ```rust
/// use lume::schema::{Column, DefaultToSql};
///
/// let string_col = Column::<String>::new("name", "users");
/// let int_col = Column::<i32>::new("age", "users");
/// let bool_col = Column::<bool>::new("active", "users");
///
/// // Set defaults
/// let string_col = string_col.default_value("John".to_string());
/// let int_col = int_col.default_value(25);
/// let bool_col = bool_col.default_value(true);
///
/// // Convert to SQL
/// assert_eq!(string_col.default_to_sql(), Some("'John'".to_string()));
/// assert_eq!(int_col.default_to_sql(), Some("25".to_string()));
/// assert_eq!(bool_col.default_to_sql(), Some("TRUE".to_string()));
/// ```
pub trait DefaultToSql {
    /// Converts the column's default value to its SQL representation.
    ///
    /// Returns `None` if the column has no default value.
    ///
    /// # Returns
    ///
    /// - `Some(String)`: The SQL representation of the default value
    /// - `None`: If no default value is set
    fn default_to_sql(&self) -> Option<String>;
}

// Implement for each column type
impl DefaultToSql for Column<String> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default()
            .map(|v| format!("'{}'", v.replace('\'', "''")))
    }
}

impl DefaultToSql for Column<i32> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<i64> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<f32> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<f64> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<bool> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| {
            if *v {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        })
    }
}
