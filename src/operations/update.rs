#![warn(missing_docs)]

//! # Query Module
//!
//! This module provides type-safe query building and execution functionality.
//! It includes the `Query<T>` struct for building and executing database queries.

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use sqlx::MySqlPool;

use crate::filter::{Filter, Filtered};
use crate::schema::{ColumnInfo, Select, Value};
use crate::{StartingSql, get_starting_sql};
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
/// use lume::filter::eq_value;
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
///         .filter(eq_value(User::name(), Value::String("John".to_string())))
///         .execute()
///         .await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Update<T: Schema + Debug> {
    /// Phantom data to maintain schema type information
    table: PhantomData<T>,
    /// List of filters to apply to the query
    filters: Vec<Box<dyn Filtered>>,
    /// Database connection pool
    conn: Arc<MySqlPool>,

    data: Vec<ColumnInfo>,
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

    pub(crate) selected_columns: Vec<&'static str>,
}

#[derive(Debug)]
pub(crate) enum JoinType {
    Left,
    Inner,
    Right,
    #[cfg(not(feature = "mysql"))]
    Full,
    Cross,
}

impl<T: Schema + Debug> Update<T> {
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
            data: T::get_all_columns(),
            conn,
        }
    }

    pub fn set(mut self, data: T) -> Self {
        self.data = T::get_all_columns();
        self
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
    /// use lume::filter::eq_value;
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
    ///         .filter(eq_value(User::name(), Value::String("John".to_string())));
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
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let users = db.query::<User, QueryUser>()
    ///         .filter(eq_value(User::name(), Value::String("John".to_string())))
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
        let sql = get_starting_sql(StartingSql::Update, T::table_name());
        let sql = Self::update_sql(sql, self.data);
        let mut params: Vec<Value> = Vec::new();
        let sql = Self::filter_sql(sql, self.filters, &mut params);

        let mut conn = self.conn.acquire().await.map_err(DatabaseError::from)?;
        let mut query = sqlx::query(&sql);
        for v in params {
            query = match v {
                Value::String(s) => query.bind(s),
                Value::Int8(i) => query.bind(i),
                Value::Int16(i) => query.bind(i),
                Value::Int32(i) => query.bind(i),
                Value::Int64(i) => query.bind(i),
                Value::UInt8(u) => query.bind(u),
                Value::Array(_arr) => {
                    eprintln!(
                        "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                    );
                    query
                }
                Value::UInt16(u) => query.bind(u),
                Value::UInt32(u) => query.bind(u),
                Value::UInt64(u) => query.bind(u),
                Value::Float32(f) => query.bind(f),
                Value::Float64(f) => query.bind(f),
                Value::Bool(b) => query.bind(b),
                Value::Null => query, // Nulls handled in SQL via IS/IS NOT
            };
        }

        let data = query
            .fetch_all(&mut *conn)
            .await
            .map_err(DatabaseError::from)?;

        // let rows = Row::from_mysql_row(data, Some(&self.joins));

        Ok(Vec::new())
    }

    pub(crate) fn update_sql(mut sql: String, data: Vec<ColumnInfo>) -> String {
        for column in data {
            sql.push_str(&format!("{} = ?", column.name));
        }
        sql
    }

    pub(crate) fn joins_sql(mut sql: String, joins: &Vec<JoinInfo>) -> String {
        if joins.is_empty() {
            return sql;
        }

        for join in joins {
            let join_type = match join.join_type {
                JoinType::Left => "LEFT JOIN",
                JoinType::Inner => "INNER JOIN",
                JoinType::Right => "RIGHT JOIN",
                #[cfg(not(feature = "mysql"))]
                JoinType::Full => "FULL JOIN",
                JoinType::Cross => "CROSS JOIN",
            };

            let join_table = &join.table_name;

            if join_type == "CROSS JOIN" {
                sql.push_str(&format!(" {} {}", join_type, join_table,));
            } else {
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
        }

        sql
    }

    fn build_filter_expr(filter: &dyn Filtered, params: &mut Vec<Value>) -> String {
        if filter.is_or_filter() || filter.is_and_filter() {
            let op = if filter.is_or_filter() { "OR" } else { "AND" };
            let Some(f1) = filter.filter1() else {
                eprintln!("Warning: Composite filter missing filter1, using tautology");
                return "1=1".to_string();
            };
            let Some(f2) = filter.filter2() else {
                eprintln!("Warning: Composite filter missing filter2, using tautology");
                return "1=1".to_string();
            };
            let left = Self::build_filter_expr(f1, params);
            let right = Self::build_filter_expr(f2, params);
            return format!("({} {} {})", left, op, right);
        }
        let Some(col1) = filter.column_one() else {
            eprintln!("Warning: Simple filter missing column_one, using tautology");
            return "1=1".to_string();
        };
        // Handle IN / NOT IN array filters
        if let Some(in_array) = filter.is_in_array() {
            let values = filter.array_values().unwrap_or(&[]);
            if values.is_empty() {
                return if in_array {
                    "1=0".to_string()
                } else {
                    "1=1".to_string()
                };
            }
            let mut placeholders: Vec<&'static str> = Vec::with_capacity(values.len());
            for v in values.iter().cloned() {
                params.push(v);
                placeholders.push("?");
            }
            let op = if in_array { "IN" } else { "NOT IN" };
            return format!("{}.{} {} ({})", col1.0, col1.1, op, placeholders.join(", "));
        }
        if let Some(value) = filter.value() {
            match value {
                Value::Null => {
                    // Special handling for NULL comparisons
                    let op = filter.filter_type();
                    let null_sql = match op {
                        crate::filter::FilterType::Eq => "IS NULL",
                        crate::filter::FilterType::Neq => "IS NOT NULL",
                        _ => {
                            // Unsupported operator with NULL; force false to avoid surprising results
                            return "1=0".to_string();
                        }
                    };
                    format!("{}.{} {}", col1.0, col1.1, null_sql)
                }
                _ => {
                    params.push(value.clone());
                    format!("{}.{} {} ?", col1.0, col1.1, filter.filter_type().to_sql())
                }
            }
        } else if let Some(col2) = filter.column_two() {
            format!(
                "{}.{} {} {}.{}",
                col1.0,
                col1.1,
                filter.filter_type().to_sql(),
                col2.0,
                col2.1
            )
        } else {
            // Fallback to a tautology if filter is malformed
            "1=1".to_string()
        }
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
            parts.push(Self::build_filter_expr(filter.as_ref(), params));
        }
        sql.push_str(&parts.join(" AND "));

        sql
    }
}
