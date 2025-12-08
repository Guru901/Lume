#![warn(missing_docs)]

//! # Database Module
//!
//! This module provides database connection and management functionality.
//! It includes the `Database` struct for managing MySQL connections and
//! executing database operations.

use sqlx::Executor;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool;
#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;
use std::{fmt::Debug, sync::Arc};

/// Error types for database operations.
pub mod error;

use crate::{
    database::error::DatabaseError,
    operations::{
        delete::Delete,
        insert::{Insert, InsertMany},
        query::Query,
        update::Update,
    },
    row::Row,
    schema::{ColumnInfo, Schema, Select, UpdateTrait},
    table::get_all_tables,
};

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
/// use lume::database::error::DatabaseError;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), DatabaseError> {
///     // Connect to database
///     let db = Database::connect("mysql://user:password@localhost/database").await?;
///     
///     // Register and create tables
///     db.register_table::<User>().await?;
///     
///     // Execute type-safe queries
///     let users = db.query::<User, SelectUser>().execute().await?;
///     
///     Ok(())
/// }
/// ```
pub struct Database {
    /// The MySQL connection pool
    #[cfg(feature = "mysql")]
    pub(crate) connection: Arc<MySqlPool>,

    #[cfg(feature = "postgres")]
    pub(crate) connection: Arc<PgPool>,

    #[cfg(feature = "sqlite")]
    pub(crate) connection: Arc<SqlitePool>,
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
    /// async fn main() -> Result<(), lume::database::error::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let query = db.query::<Users, SelectUsers>();
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
    /// async fn main() -> Result<(), lume::database::error::DatabaseError> {
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

    /// Creates a new type-safe delete operation for the specified schema type.
    ///
    /// # Arguments
    ///
    /// - `T`: The schema type to delete from (must implement [`Schema`] + [`Debug`])
    ///
    /// # Returns
    ///
    /// A [`Delete<T>`] instance that can be used to build and execute a delete query.
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
    /// async fn main() -> Result<(), lume::database::error::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///
    ///     db.delete::<Users>()
    ///         .filter(lume::filter::eq_value(Users::name(), "guru"))
    ///         .execute()
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn delete<T: Schema + Debug>(&self) -> Delete<T> {
        Delete::new(Arc::clone(&self.connection))
    }

    /// Creates a new type-safe update operation for the specified schema type.
    ///
    /// # Arguments
    ///
    /// - `T`: The schema type to update (must implement [`Schema`] + [`Debug`])
    ///
    /// # Returns
    ///
    /// An [`Update<T>`] instance that can be used to build and execute an update query.
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
    ///         age: i32,
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::error::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///
    ///     db.update::<Users, UpdateUsers>()
    ///         .set(UpdateUsers {
    ///             age: Some(2),
    ///             ..Default::default()
    ///         })
    ///         .filter(lume::filter::eq_value(Users::name(), "guru"))
    ///         .execute()
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn update<T: Schema + Debug, U: UpdateTrait + Debug>(&self) -> Update<T, U> {
        Update::new(Arc::clone(&self.connection))
    }

    /// Creates a new type-safe insert-many for the specified schema type.
    ///
    /// Accepts any iterable of schema values, enabling println!-style multiple values.
    pub fn insert_many<T: Schema + Debug, I>(&self, data: I) -> InsertMany<T>
    where
        I: IntoIterator<Item = T>,
    {
        InsertMany::new(data.into_iter().collect(), Arc::clone(&self.connection))
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
    /// use lume::database::error::DatabaseError;
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
        let conn = self.connection.acquire().await;

        if let Err(e) = conn {
            return Err(DatabaseError::ConnectionError(e));
        }

        let mut conn = conn.unwrap();

        let rows = conn.fetch_all(sql).await;

        if let Err(e) = rows {
            return Err(DatabaseError::QueryError(e.to_string()));
        }

        let rows = rows.unwrap();

        #[cfg(feature = "mysql")]
        let rows = Row::from_mysql_row(rows, None);

        #[cfg(feature = "postgres")]
        let rows = Row::from_postgres_row(rows, None);

        #[cfg(feature = "sqlite")]
        let rows = Row::from_sqlite_row(rows, None);

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
    /// use lume::database::error::DatabaseError;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DatabaseError> {
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
                .map_err(|e| DatabaseError::ExecutionError(e.to_string()))?;
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

        #[allow(unused_mut)]
        let mut statements: Vec<String> =
            tables.iter().map(|table| table.to_create_sql()).collect();

        #[cfg(feature = "postgres")]
        {
            statements = statements
                .into_iter()
                .map(Self::adapt_sql_for_postgres)
                .collect();
        }

        #[cfg(feature = "sqlite")]
        {
            statements = statements
                .into_iter()
                .map(Self::adapt_sql_for_sqlite)
                .collect();
        }

        statements.join("\n\n")
    }

    #[cfg(feature = "postgres")]
    fn adapt_sql_for_postgres(sql: String) -> String {
        // Basic textual normalization to keep shared schema metadata working
        // with PostgreSQL-specific syntax expectations.
        const REPLACEMENTS: &[(&str, &str)] = &[
            ("TINYINT UNSIGNED", "SMALLINT"),
            ("TINYINT", "SMALLINT"),
            ("SMALLINT UNSIGNED", "INTEGER"),
            ("INTEGER UNSIGNED", "BIGINT"),
            ("BIGINT UNSIGNED", "BIGINT"),
            ("DOUBLE", "DOUBLE PRECISION"),
            ("DEFAULT (UUID())", "gen_random_uuid()"),
        ];

        let mut converted = sql;

        // Replace VARCHAR or any type with UUID if default is (UUID())
        if converted.contains("DEFAULT (UUID())") {
            // This regex replaces '<type> DEFAULT (UUID())' with 'UUID DEFAULT gen_random_uuid()'
            let uuid_type_pattern =
                regex::Regex::new(r"(\s*)([A-Z]+(?:\([0-9, ]+\))?)\s+DEFAULT \(UUID\(\)\)")
                    .unwrap();
            converted = uuid_type_pattern
                .replace_all(&converted, |caps: &regex::Captures| {
                    format!("{}UUID DEFAULT gen_random_uuid()", &caps[1])
                })
                .to_string();
        }
        for (mysql, postgres) in REPLACEMENTS {
            converted = converted.replace(mysql, postgres);
        }

        // PostgreSQL uses identity columns instead of AUTO_INCREMENT.
        converted = converted.replace("AUTO_INCREMENT", "GENERATED BY DEFAULT AS IDENTITY");

        converted
    }

    #[cfg(feature = "sqlite")]
    fn adapt_sql_for_sqlite(sql: String) -> String {
        // Basic textual normalization to keep shared schema metadata working
        // with PostgreSQL-specific syntax expectations.
        const REPLACEMENTS: &[(&str, &str)] =
            &[("DEFAULT (UUID())", "DEFAULT (lower(hex(randomblob(16))))")];

        let mut converted = sql;

        for (mysql, sqlite) in REPLACEMENTS {
            converted = converted.replace(mysql, sqlite);
        }

        converted
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
    /// use lume::database::error::DatabaseError;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DatabaseError> {
    ///     let db = Database::connect("mysql://user:password@localhost/mydb").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(url: &str) -> Result<Database, DatabaseError> {
        #[cfg(feature = "mysql")]
        let conn = MySqlPool::connect(url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e))?;

        #[cfg(feature = "postgres")]
        let conn = PgPool::connect(url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e))?;

        #[cfg(feature = "sqlite")]
        let conn = SqlitePool::connect(url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e))?;

        Ok(Database {
            connection: Arc::new(conn),
        })
    }
}
