//! # Table Module
//!
//! This module provides table registry functionality and the `TableDefinition` trait
//! for managing database table metadata and SQL generation.

use std::sync::{Mutex, OnceLock};

use crate::schema::{ColumnInfo, Schema, SchemaWrapper};

/// Global table registry for storing all registered tables
static TABLE_REGISTRY: OnceLock<Mutex<Vec<Box<dyn TableDefinition>>>> = OnceLock::new();

/// Trait for defining database table metadata and SQL generation.
///
/// This trait provides the interface for table definitions, including
/// table names, column metadata, and SQL generation for CREATE TABLE statements.
///
/// # Features
///
/// - **Table Metadata**: Access to table names and column information
/// - **SQL Generation**: Automatic CREATE TABLE statement generation
/// - **Index Creation**: Support for automatic index creation
/// - **Constraint Handling**: Primary keys, unique constraints, and NOT NULL
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
/// use lume::table::TableDefinition;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key().not_null()],
///         name: String [not_null()],
///         email: String [unique()],
///     }
/// }
///
/// User::ensure_registered();
/// let tables = lume::table::get_all_tables();
/// let user_table = tables.iter().find(|t| t.table_name() == "User").unwrap();
///
/// println!("CREATE TABLE SQL: {}", user_table.to_create_sql());
/// ```
pub trait TableDefinition: Send + Sync {
    /// Returns the name of this table.
    fn table_name(&self) -> &'static str;

    /// Returns metadata for all columns in this table.
    fn get_columns(&self) -> Vec<ColumnInfo>;

    /// Generates the CREATE TABLE SQL statement for this table.
    ///
    /// This includes all columns, constraints, and indexes.
    fn to_create_sql(&self) -> String;

    /// Creates a boxed clone of this table definition.
    fn clone_box(&self) -> Box<dyn TableDefinition>;
}

/// Registers a schema type in the global table registry.
///
/// This function is idempotent - calling it multiple times with the same
/// schema type will not create duplicate entries.
///
/// # Arguments
///
/// - `T`: The schema type to register (must implement `Schema + Send + Sync + 'static`)
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
/// }
///
/// // Register the table
/// lume::table::register_table::<User>();
///
/// // Multiple calls are safe
/// lume::table::register_table::<User>(); // No duplicate created
/// ```
pub fn register_table<T: Schema + Send + Sync + 'static>() {
    let registry = TABLE_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
    let mut tables = registry.lock().unwrap();

    // Check if table already registered; return early to keep idempotent
    let table_name = T::table_name();
    let already_exists = tables.iter().any(|t| t.table_name() == table_name);
    if already_exists {
        return;
    }

    tables.push(Box::new(SchemaWrapper::<T>::new()));
}

/// Returns all registered tables as a vector of boxed `TableDefinition` instances.
///
/// # Returns
///
/// A vector containing all registered table definitions
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
/// }
///
/// define_schema! {
///     Post {
///         id: i32 [primary_key()],
///         title: String [not_null()],
///     }
/// }
///
/// User::ensure_registered();
/// Post::ensure_registered();
///
/// let tables = lume::table::get_all_tables();
/// assert_eq!(tables.len(), 2);
///
/// for table in tables {
///     println!("Table: {}", table.table_name());
///     println!("SQL: {}", table.to_create_sql());
/// }
/// ```
pub fn get_all_tables() -> Vec<Box<dyn TableDefinition>> {
    let registry = TABLE_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
    let tables = registry.lock().unwrap();
    tables.iter().map(|t| t.clone_box()).collect()
}
