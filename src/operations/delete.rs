use std::{fmt::Debug, marker::PhantomData, sync::Arc};

#[cfg(feature = "mysql")]
use sqlx::MySqlPool;
#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;

use crate::{
    database::error::DatabaseError,
    filter::Filtered,
    schema::{Schema, Value},
};

use crate::helpers::{StartingSql, bind_value, build_filter_expr, get_starting_sql};

/// Represents a SQL DELETE operation for a given table.
///
/// The `Delete<T>` struct is used to construct and execute a type-safe
/// SQL DELETE statement for the table represented by the schema type `T`.
/// It allows you to specify filter conditions to control which records
/// are deleted from the database.
///
/// # Type Parameters
///
/// * `T` - The schema type representing the table to delete from. This type must implement [`Schema`].
///
/// # Fields
///
/// - `table`: Marker for the schema type `T`. Used for type safety and SQL generation.
/// - `filters`: A list of filter conditions (implementing [`Filtered`]) to restrict which rows are deleted.
/// - `conn`: The database connection pool used to execute the delete operation.
///
/// # Example
///
/// ```no_run
/// use lume::define_schema;
/// use lume::database::Database;
/// use lume::filter::eq_value;
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
/// async fn main() -> Result<(), lume::database::error::DatabaseError> {
///     let db = Database::connect("mysql://...").await?;
///     db.delete::<User>()
///         .filter(eq_value(User::name(), "Alice"))
///         .execute()
///         .await?;
///     Ok(())
/// }
/// ```
pub struct Delete<T> {
    /// Marker for the schema type `T`.
    table: PhantomData<T>,
    /// List of filters to apply to the delete query.
    filters: Vec<Box<dyn Filtered>>,
    /// Database connection pool.
    #[cfg(feature = "mysql")]
    conn: Arc<MySqlPool>,

    #[cfg(feature = "postgres")]
    conn: Arc<PgPool>,

    #[cfg(feature = "sqlite")]
    conn: Arc<SqlitePool>,
}

impl<T: Schema + Debug> Delete<T> {
    /// Creates a new [`Delete`] operation for the given data and connection.
    ///
    /// # Arguments
    ///
    /// * `data` - The record to delete.
    /// * `conn` - The database connection pool.
    ///
    /// # Returns
    ///
    /// An [`Delete`] instance ready for execution.
    #[cfg(feature = "mysql")]
    pub fn new(conn: Arc<MySqlPool>) -> Self {
        Self {
            table: PhantomData,
            conn,
            filters: Vec::new(),
        }
    }

    /// Creates a new [`Delete`] operation for the given data and connection.
    ///
    /// # Arguments
    ///
    /// * `data` - The record to delete.
    /// * `conn` - The database connection pool.
    ///
    /// # Returns
    ///
    /// An [`Delete`] instance ready for execution.
    #[cfg(feature = "postgres")]
    pub fn new(conn: Arc<PgPool>) -> Self {
        Self {
            table: PhantomData,
            conn,
            filters: Vec::new(),
        }
    }

    /// Creates a new [`Delete`] operation for the given data and connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - The database connection pool.
    ///
    /// # Returns
    ///
    /// An [`Delete`] instance ready for execution.
    #[cfg(feature = "sqlite")]
    pub fn new(conn: Arc<SqlitePool>) -> Self {
        Self {
            table: PhantomData,
            conn,
            filters: Vec::new(),
        }
    }

    /// Adds a filter condition to the delete operation.
    ///
    /// This method allows you to specify a filter that determines which records
    /// will be deleted from the table. You can chain multiple calls to `filter`
    /// to combine multiple filter conditions (e.g., using logical AND/OR).
    ///
    /// # Arguments
    ///
    /// * `filter` - A filter implementing the [`Filtered`] trait, representing the condition to apply.
    ///
    /// # Returns
    ///
    /// Returns a new [`Delete`] instance with the filter added, allowing for method chaining.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::define_schema;
    /// use lume::database::Database;
    /// use lume::filter::eq_value;
    /// use lume::schema::Schema;
    /// use lume::schema::ColumnInfo;
    ///
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::error::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     db.delete::<User>()
    ///         .filter(eq_value(User::name(), Value::String("John".to_string())))
    ///         .execute()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Filtered + 'static,
    {
        self.filters.push(Box::new(filter));
        self
    }

    /// Executes the delete operation.
    ///
    /// This method builds and executes the SQL DELETE query, removing records
    /// that match the specified filters.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the delete operation was successful
    /// - `Err(DatabaseError)`: If there was an error executing the query
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lume::define_schema;
    /// use lume::database::Database;
    /// use lume::filter::Filter;
    /// use lume::schema::{Schema, ColumnInfo};
    /// use lume::filter::eq_value;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::error::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     db.delete::<User>()
    ///         .filter(eq_value(User::name(), Value::String("John".to_string())))
    ///         .execute()
    ///         .await?;
    ///
    ///     println!("Users deleted successfully");
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute(self) -> Result<(), DatabaseError> {
        let sql = get_starting_sql(StartingSql::Delete, T::table_name());

        let mut params: Vec<Value> = Vec::new();

        let sql = Self::filter_sql(sql, self.filters, &mut params);

        let mut conn = self
            .conn
            .acquire()
            .await
            .map_err(|e| DatabaseError::ConnectionError(e))?;
        let mut query = sqlx::query(&sql);
        for v in params {
            query = bind_value(query, v);
        }

        query
            .execute(&mut *conn)
            .await
            .map_err(|e| DatabaseError::ExecutionError(e.to_string()))?;

        Ok(())
    }

    pub(crate) fn filter_sql(
        mut sql: String,
        filters: Vec<Box<dyn Filtered>>,
        params: &mut Vec<Value>,
    ) -> String {
        if filters.is_empty() {
            return sql;
        }

        sql.push_str(" WHERE ");
        let mut parts: Vec<String> = Vec::with_capacity(filters.len());
        for filter in &filters {
            parts.push(build_filter_expr(filter.as_ref(), params));
        }
        sql.push_str(&parts.join(" AND "));

        sql
    }
}
