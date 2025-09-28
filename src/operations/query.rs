#![warn(missing_docs)]

//! # Query Module
//!
//! This module provides type-safe query building and execution functionality.
//! It includes the `Query<T>` struct for building and executing database queries.

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use sqlx::MySqlPool;

use crate::filter::Filter;
use crate::schema::{ColumnInfo, Select};
use crate::{StartingSql, filter_sql, get_starting_sql, joins_sql, select_sql};
use crate::{database::DatabaseError, row::Row, schema::Schema};

/// A type-safe query builder for database operations.
///
/// The `Query<T, S>` struct provides a fluent interface for building and executing
/// database queries with compile-time type safety.
///
/// # Type Parameters
///
/// - `T`: The schema type to query (must implement `Schema + Debug`)
/// - `S`: The selection type for column specification (must implement `Select + Debug`)
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
/// use lume::schema::{Schema, ColumnInfo};
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
///     let users = db.query::<User, QueryUser>()
///         .filter(eq(User::name(), Value::String("John".to_string())))
///         .execute()
///         .await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Query<T, S> {
    /// Phantom data to maintain schema type information
    table: PhantomData<T>,
    /// List of filters to apply to the query
    filters: Vec<Filter>,
    /// Database connection pool
    conn: Arc<MySqlPool>,

    select: Option<S>,

    joins: Vec<JoinInfo>,
}

/// Information about a join operation
#[derive(Debug)]
pub(crate) struct JoinInfo {
    /// The table to join
    pub(crate) table_name: String,
    /// The join condition (column-to-column comparison)
    pub(crate) condition: Filter,

    pub(crate) join_type: JoinType,

    pub(crate) columns: Vec<ColumnInfo>,
}

#[derive(Debug)]
pub(crate) enum JoinType {
    Left,
}

impl<T: Schema + Debug, S: Select + Debug> Query<T, S> {
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
            select: None,
            joins: Vec::new(),
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
    /// use lume::schema::{Schema, ColumnInfo};
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
    ///     let query = db.query::<User, QueryUser>()
    ///         .filter(eq(User::name(), Value::String("John".to_string())));
    ///     Ok(())
    /// }
    /// ```
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Specifies which columns to select in the query.
    ///
    /// This method accepts a selection schema that determines which columns
    /// will be included in the SELECT clause of the SQL query.
    ///
    /// # Arguments
    ///
    /// - `select_schema`: The selection schema specifying which columns to include
    ///
    /// # Returns
    ///
    /// The query builder instance for method chaining

    pub fn select(mut self, select_schema: S) -> Self {
        self.select = Some(select_schema);
        self
    }

    /// Adds a left join to the query.
    ///
    /// This method is currently a placeholder for future join functionality.
    ///
    /// # Arguments
    ///
    /// - `filter`: The join condition (currently unused)
    ///
    /// # Returns
    ///
    /// The query builder instance for method chaining
    pub fn left_join<LeftJoinSchema: Schema + Debug>(mut self, filter: Filter) -> Self {
        self.joins.push(JoinInfo {
            table_name: LeftJoinSchema::table_name().to_string(),
            condition: filter,
            join_type: JoinType::Left,
            columns: LeftJoinSchema::get_all_columns(),
        });

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
    /// use lume::schema::{Schema, ColumnInfo};
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
    ///     let users = db.query::<User, QueryUser>()
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
        let sql = get_starting_sql(StartingSql::Select);
        let sql = select_sql(sql, self.select, T::table_name());
        let sql = joins_sql(sql, &self.joins);
        let sql = filter_sql(sql, self.filters);

        println!("SQL: {}", sql);

        let mut conn = self.conn.acquire().await.unwrap();

        let data = sqlx::query(&sql).fetch_all(&mut *conn).await.unwrap();

        let rows = Row::from_mysql_row(data, Some(&self.joins));

        Ok(rows)
    }
}
