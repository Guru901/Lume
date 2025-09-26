//! # Query Module
//!
//! This module provides type-safe query building and execution functionality.
//! It includes the `Query<T>` struct for building and executing database queries.

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use sqlx::MySqlPool;

use crate::{database::DatabaseError, row::Row, schema::Schema};
use crate::{filter::Filter, schema::Value};

/// A type-safe query builder for database operations.
///
/// The `Query<T>` struct provides a fluent interface for building and executing
/// database queries with compile-time type safety.
///
/// # Type Parameters
///
/// - `T`: The schema type to query (must implement `Schema + Debug`)
///
/// # Features
///
/// - **Type Safety**: Compile-time type checking for all query operations
/// - **Fluent Interface**: Chainable methods for building complex queries
/// - **Filtering**: Support for WHERE clause conditions
/// - **MySQL Integration**: Built-in support for MySQL database operations
///
/// # Example
///
/// ```no_run
/// use lume::define_schema;
/// use lume::database::Database;
/// use lume::filter::Filter;
/// use lume::schema::{Schema, ColumnInfo, Value};
/// use lume::filter::eq;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), lume::database::DatabaseError> {
///     let db = Database::connect("mysql://...").await?;
///     let users = db.query::<User>()
///         .filter(eq(User::name(), Value::String("John".to_string())))
///         .execute()
///         .await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Query<T> {
    /// Phantom data to maintain schema type information
    table: PhantomData<T>,
    /// List of filters to apply to the query
    filters: Vec<Filter>,
    /// Database connection pool
    conn: Arc<MySqlPool>,
}

impl<T: Schema + Debug> Query<T> {
    /// Creates a new query builder for the specified schema type.
    ///
    /// # Arguments
    ///
    /// - `conn`: The database connection pool
    ///
    /// # Returns
    ///
    /// A new `Query<T>` instance ready for building queries
    pub(crate) fn new(conn: Arc<MySqlPool>) -> Self {
        Self {
            table: PhantomData,
            filters: Vec::new(),
            conn,
        }
    }

    /// Adds a filter condition to the query.
    ///
    /// This method allows chaining multiple filter conditions to build
    /// complex WHERE clauses. All filters are combined with AND logic.
    ///
    /// # Arguments
    ///
    /// - `filter`: The filter condition to add
    ///
    /// # Returns
    ///
    /// The query builder instance for method chaining
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::define_schema;
    /// use lume::database::Database;
    /// use lume::filter::Filter;
    /// use lume::schema::{Schema, ColumnInfo, Value};
    /// use lume::filter::eq;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///         age: i32,
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let query = db.query::<User>()
    ///         .filter(eq(User::name(), Value::String("John".to_string())));
    ///     Ok(())
    /// }
    /// ```
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Specifies that this is a SELECT query.
    ///
    /// This method is currently a no-op but is included for API completeness
    /// and future extensibility.
    ///
    /// # Returns
    ///
    /// The query builder instance for method chaining
    pub fn select(self) -> Self {
        self
    }

    /// Executes the query and returns the results.
    ///
    /// This method builds and executes the SQL query, returning type-safe
    /// row objects that can be used to access column values.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Row<T>>)`: A vector of type-safe row objects
    /// - `Err(DatabaseError)`: If there was an error executing the query
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::define_schema;
    /// use lume::database::Database;
    /// use lume::filter::Filter;
    /// use lume::schema::{Schema, ColumnInfo, Value};
    /// use lume::filter::eq;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let users = db.query::<User>()
    ///         .filter(eq(User::name(), Value::String("John".to_string())))
    ///         .execute()
    ///         .await?;
    ///
    ///     for user in users {
    ///         let name: Option<String> = user.get(User::name());
    ///         println!("User: {:?}", name);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute(self) -> Result<Vec<Row<T>>, DatabaseError> {
        let mut sql = format!("SELECT * FROM {}", T::table_name());
        let mut conn = self.conn.acquire().await.unwrap();

        if !self.filters.is_empty() {
            let filter_sql = format!(" WHERE ");
            sql.push_str(&filter_sql);

            for (i, filter) in self.filters.iter().enumerate() {
                match &filter.value {
                    Value::String(_) => {
                        let filter_sql = format!(
                            "{} {} '{}' {}",
                            filter.column_name,
                            filter.filter_type.to_sql(),
                            filter.value,
                            if i == self.filters.len() - 1 {
                                ""
                            } else {
                                " AND "
                            }
                        );
                        sql.push_str(&filter_sql);
                    }
                    _ => {
                        let filter_sql = format!(
                            "{} {} {} {}",
                            filter.column_name,
                            filter.filter_type.to_sql(),
                            filter.value,
                            if i == self.filters.len() - 1 {
                                ""
                            } else {
                                " AND "
                            }
                        );

                        sql.push_str(&filter_sql);
                    }
                }
            }
        }

        let data = sqlx::query(&sql).fetch_all(&mut *conn).await.unwrap();
        let rows = Row::from_mysql_row(data);

        Ok(rows)
    }
}
