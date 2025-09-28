#![warn(missing_docs)]

//! # Database Module
//!
//! This module provides database connection and management functionality.
//! It includes the `Database` struct for managing MySQL connections and
//! executing database operations.

use sqlx::Executor;
use std::{fmt::Debug, sync::Arc};

use crate::{
    operations::{insert::Insert, query::Query},
    row::Row,
    schema::{ColumnInfo, Schema, Select},
    table::get_all_tables,
};
use sqlx::MySqlPool;

/// A database connection manager that provides type-safe access to MySQL databases.
///
/// The `Database` struct manages a connection pool and provides methods for
/// executing queries, registering tables, and managing database schema.
///
/// # Features
///
/// - **Connection Pooling**: Efficient management of database connections
/// - **Type-Safe Queries**: Compile-time type checking for all database operations
/// - **Schema Management**: Automatic table creation and migration support
/// - **Error Handling**: Comprehensive error handling with custom error types
///
/// # Example
///
/// ```no_run
/// use lume::database::Database;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Connect to database
///     let db = Database::connect("mysql://user:password@localhost/database").await?;
///     
///     // Register and create tables
///     db.register_table::<User>().await?;
///     
///     // Execute type-safe queries
///     let users = db.query::<User, QueryUser>().execute().await?;
///     
///     Ok(())
/// }
/// ```
pub struct Database {
    /// The MySQL connection pool
    connection: Arc<MySqlPool>,
}

impl Database {
    /// Creates a new type-safe query builder for the specified schema type.
    ///
    /// # Arguments
    ///
    /// - `T`: The schema type to query (must implement `Schema + Debug`)
    ///
    /// # Returns
    ///
    /// A `Query<T>` instance that can be used to build and execute database queries
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::database::Database;
    /// use lume::define_schema;
    /// use lume::schema::Schema;
    /// use lume::schema::ColumnInfo;
    ///
    /// define_schema! {
    ///     Users {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let query = db.query::<Users, QueryUsers>();
    ///     Ok(())
    /// }
    /// ```
    pub fn query<T: Schema + Debug, S: Select + Debug>(&self) -> Query<T, S> {
        Query::new(Arc::clone(&self.connection))
    }

    /// Creates a new type-safe insert for the specified schema type.
    ///
    /// # Arguments
    ///
    /// - `T`: The schema type to insert (must implement `Schema + Debug`)
    ///
    /// # Returns
    ///
    /// A `Insert<T>` instance that can be used to insert data into the database
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::database::Database;
    /// use lume::define_schema;
    /// use lume::schema::Schema;
    /// use lume::schema::ColumnInfo;
    ///
    /// define_schema! {
    ///     Users {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///
    ///     db.insert(Users {
    ///         id: 1,
    ///         name: "guru".to_string(),
    ///     })
    ///     .execute()
    ///     .await
    ///     .unwrap();
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn insert<T: Schema + Debug>(&self, data: T) -> Insert<T> {
        Insert::new(data, Arc::clone(&self.connection))
    }

    /// Executes a raw SQL query and returns typed rows.
    ///
    /// # Safety
    ///
    /// This method bypasses the query builder's type safety. Ensure the SQL
    /// query returns columns that match the schema type `T`.
    ///
    /// # Arguments
    ///
    /// - `sql`: The raw SQL query to execute
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Row<T>>)`: A vector of typed rows
    /// - `Err(DatabaseError)`: If there was an error executing the query
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::database::Database;
    /// use lume::database::DatabaseError;
    /// use lume::define_schema;
    /// use lume::schema::ColumnInfo;
    /// use lume::schema::Schema;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key().not_null()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let users = db.sql::<User>("SELECT * FROM User WHERE age > 18").await?;
    ///
    ///     Ok(())
    /// }
    /// ```

    pub async fn sql<T: Schema + Debug>(&self, sql: &str) -> Result<Vec<Row<T>>, DatabaseError> {
        let mut conn = self.connection.acquire().await?;
        let rows = conn.fetch_all(sql).await?;
        let rows = Row::from_mysql_row::<T>(rows, None);
        Ok(rows)
    }

    /// Registers a schema type and creates its corresponding database table.
    ///
    /// This method ensures the schema is registered and then executes the
    /// CREATE TABLE statements to create the table in the database.
    ///
    /// # Arguments
    ///
    /// - `T`: The schema type to register (must implement `Schema`)
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the table was successfully created
    /// - `Err(DatabaseError)`: If there was an error creating the table
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::database::Database;
    /// use lume::define_schema;
    /// use lume::schema::Schema;
    /// use lume::schema::ColumnInfo;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     db.register_table::<User>().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn register_table<T: Schema>(&self) -> Result<(), DatabaseError> {
        T::ensure_registered();
        let sql = Database::generate_migration_sql();
        for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(stmt)
                .execute(&*self.connection)
                .await
                .map_err(DatabaseError)?;
        }
        Ok(())
    }

    /// Generates SQL migration statements for all registered tables.
    ///
    /// This method creates CREATE TABLE statements for all tables that have
    /// been registered in the global table registry.
    ///
    /// # Returns
    ///
    /// A string containing all CREATE TABLE statements, separated by newlines
    pub(crate) fn generate_migration_sql() -> String {
        let tables = get_all_tables();
        tables
            .iter()
            .map(|table| table.to_create_sql())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Retrieves column information for a specific table.
    ///
    /// # Arguments
    ///
    /// - `table_name`: The name of the table to get information for
    ///
    /// # Returns
    ///
    /// - `Some(Vec<ColumnInfo>)`: Column information if the table exists
    /// - `None`: If the table is not registered
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::database::Database;
    /// use lume::define_schema;
    /// use lume::schema::Schema;
    /// use lume::schema::ColumnInfo;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// User::ensure_registered();
    /// let columns = Database::get_table_info("User");
    /// if let Some(cols) = columns {
    ///     println!("User table has {} columns", cols.len());
    /// }
    /// ```
    pub fn get_table_info(table_name: &str) -> Option<Vec<ColumnInfo>> {
        let tables = get_all_tables();
        tables
            .iter()
            .find(|table| table.table_name() == table_name)
            .map(|table| table.get_columns())
    }

    /// Returns a list of all registered table names.
    ///
    /// # Returns
    ///
    /// A vector containing the names of all registered tables
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::database::Database;
    /// use lume::define_schema;
    /// use lume::schema::Schema;
    /// use lume::schema::ColumnInfo;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///     }
    /// }
    ///
    /// User::ensure_registered();
    /// let tables = Database::list_tables();
    /// assert!(tables.contains(&"User".to_string()));
    /// ```
    pub fn list_tables() -> Vec<String> {
        let tables = get_all_tables();
        tables
            .iter()
            .map(|table| table.table_name().to_string())
            .collect()
    }

    /// Establishes a connection to a MySQL database.
    ///
    /// # Arguments
    ///
    /// - `url`: The MySQL connection URL (e.g., "mysql://user:password@localhost/database")
    ///
    /// # Returns
    ///
    /// - `Ok(Database)`: If the connection was successful
    /// - `Err(DatabaseError)`: If there was an error connecting
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::database::Database;
    /// use lume::database::DatabaseError;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DatabaseError> {
    ///     let db = Database::connect("mysql://user:password@localhost/mydb").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(url: &str) -> Result<Database, DatabaseError> {
        let conn = MySqlPool::connect(url).await.map_err(DatabaseError)?;
        Ok(Database {
            connection: Arc::new(conn),
        })
    }
}

/// Error type for database operations.
///
/// This error type wraps SQLx errors and provides a unified error interface
/// for all database operations in Lume.
///
/// # Example
///
/// ```no_run
/// use lume::database::{Database, DatabaseError};
///
/// async fn example() -> Result<(), DatabaseError> {
///     let db = Database::connect("mysql://invalid_url").await?;
///     // Handle database operations...
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct DatabaseError(sqlx::Error);

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database error: {}", self.0)
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        DatabaseError(err)
    }
}
