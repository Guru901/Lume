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
pub struct Query<T, S> {
    /// Phantom data to maintain schema type information
    table: PhantomData<T>,
    /// List of filters to apply to the query
    filters: Vec<Box<dyn Filtered>>,
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
    /// This method joins the specified schema table to the current query using a LEFT JOIN.
    /// All records from the left table (current query) are returned, along with matching
    /// records from the right table (joined table).
    ///
    /// # Arguments
    ///
    /// - `filter`: The join condition specifying how tables should be joined
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
    /// use lume::filter::eq_column;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    ///
    ///     Post {
    ///         id: i32 [primary_key()],
    ///         user_id: i32,
    ///         title: String,
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let results = db.query::<User, QueryUser>()
    ///         .left_join::<Post, QueryPost>(eq_column(User::id(), Post::user_id()), QueryPost { title: true, ..Default::default() })
    ///         .execute()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn left_join<LeftJoinSchema: Schema + Debug, LeftJoinSchemaSelect: Select + Debug>(
        mut self,
        filter: Filter,
        select_schema: LeftJoinSchemaSelect,
    ) -> Self {
        self.joins.push(JoinInfo {
            table_name: LeftJoinSchema::table_name().to_string(),
            condition: filter,
            join_type: JoinType::Left,
            columns: LeftJoinSchema::get_all_columns(),
            selected_columns: select_schema.get_selected(),
        });

        self
    }

    /// Adds an inner join to the query.
    ///
    /// This method joins the specified schema table to the current query using an INNER JOIN.
    /// Only records that have matching values in both tables are returned.
    ///
    /// # Arguments
    ///
    /// - `filter`: The join condition specifying how tables should be joined
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
    /// use lume::filter::eq_column;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    ///
    ///     Post {
    ///         id: i32 [primary_key()],
    ///         user_id: i32,
    ///         title: String,
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let results = db.query::<User, QueryUser>()
    ///         .inner_join::<Post, QueryPost>(eq_column(User::id(), Post::user_id()), QueryPost { title: true, ..Default::default() })
    ///         .execute()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn inner_join<InnerJoinSchema: Schema + Debug, InnerJoinSchemaSelect: Select + Debug>(
        mut self,
        filter: Filter,
        select_schema: InnerJoinSchemaSelect,
    ) -> Self {
        self.joins.push(JoinInfo {
            table_name: InnerJoinSchema::table_name().to_string(),
            condition: filter,
            join_type: JoinType::Inner,
            columns: InnerJoinSchema::get_all_columns(),
            selected_columns: select_schema.get_selected(),
        });

        self
    }

    /// Adds a right join to the query.
    ///
    /// This method joins the specified schema table to the current query using a RIGHT JOIN.
    /// All records from the right table (joined table) are returned, along with matching
    /// records from the left table (current query).
    ///
    /// # Arguments
    ///
    /// - `filter`: The join condition specifying how tables should be joined
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
    /// use lume::filter::eq_column;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    ///
    ///     Post {
    ///         id: i32 [primary_key()],
    ///         user_id: i32,
    ///         title: String,
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let results = db.query::<User, QueryUser>()
    ///         .right_join::<Post, QueryPost>(eq_column(User::id(), Post::user_id()), QueryPost { title: true, ..Default::default() })
    ///         .execute()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn right_join<RightJoinSchema: Schema + Debug, RightJoinSchemaSelect: Select + Debug>(
        mut self,
        filter: Filter,
        select_schema: RightJoinSchemaSelect,
    ) -> Self {
        self.joins.push(JoinInfo {
            table_name: RightJoinSchema::table_name().to_string(),
            condition: filter,
            join_type: JoinType::Right,
            columns: RightJoinSchema::get_all_columns(),
            selected_columns: select_schema.get_selected(),
        });

        self
    }

    #[cfg(not(feature = "mysql"))]
    /// Adds a full outer join to the query.
    ///
    /// This method joins the specified schema table to the current query using a FULL OUTER JOIN.
    /// All records from both tables are returned, with NULL values for non-matching records.
    ///
    /// # Arguments
    ///
    /// - `filter`: The join condition specifying how tables should be joined
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
    /// use lume::filter::eq_column;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    ///
    ///     Post {
    ///         id: i32 [primary_key()],
    ///         user_id: i32,
    ///         title: String,
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let results = db.query::<User, QueryUser>()
    ///         .full_join::<Post>(eq_column(User::id(), Post::user_id()))
    ///         .execute()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn full_join<FullJoinSchema: Schema + Debug, FullJoinSchemaSelect: Select + Debug>(
        mut self,
        filter: Filter,
        select_schema: FullJoinSchemaSelect,
    ) -> Self {
        self.joins.push(JoinInfo {
            table_name: FullJoinSchema::table_name().to_string(),
            condition: filter,
            join_type: JoinType::Full,
            columns: FullJoinSchema::get_all_columns(),
            selected_columns: select_schema.get_selected(),
        });

        self
    }

    /// Adds a cross join to the query.
    ///
    /// This method joins the specified schema table to the current query using a CROSS JOIN.
    /// This produces a Cartesian product of all records from both tables.
    ///
    /// # Arguments
    ///
    /// - `filter`: The join condition (note: cross joins typically don't use conditions)
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
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    ///
    ///     Post {
    ///         id: i32 [primary_key()],
    ///         user_id: i32,
    ///         title: String,
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lume::database::DatabaseError> {
    ///     let db = Database::connect("mysql://...").await?;
    ///     let results = db.query::<User, QueryUser>()
    ///         .cross_join::<Post, QueryPost>(QueryPost { title: true, ..Default::default() })
    ///         .execute()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn cross_join<CrossJoinSchema: Schema + Debug, CrossJoinSchemaSelect: Select + Debug>(
        mut self,
        select_schema: CrossJoinSchemaSelect,
    ) -> Self {
        self.joins.push(JoinInfo {
            table_name: CrossJoinSchema::table_name().to_string(),
            condition: Filter::default(),
            join_type: JoinType::Cross,
            columns: CrossJoinSchema::get_all_columns(),
            selected_columns: select_schema.get_selected(),
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
        let sql = get_starting_sql(StartingSql::Select, T::table_name());
        let sql = Self::select_sql(sql, self.select, T::table_name(), &self.joins);
        let sql = Self::joins_sql(sql, &self.joins);
        let sql = Self::filter_sql(sql, self.filters);

        println!("SQL: {sql}");

        let mut conn = self.conn.acquire().await.map_err(DatabaseError::from)?;
        let data = sqlx::query(&sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(DatabaseError::from)?;

        let rows = Row::from_mysql_row(data, Some(&self.joins));

        Ok(rows)
    }

    pub(crate) fn select_sql(
        mut sql: String,
        select: Option<S>,
        table_name: &str,
        joins: &Vec<JoinInfo>,
    ) -> String {
        if select.is_some() {
            sql.push_str(&select.unwrap().get_selected().join(", "));
        } else {
            sql.push_str("*");
        }

        if !joins.is_empty() {
            for join in joins {
                for column in &join.selected_columns {
                    sql.push_str(&format!(", {}", column));
                }
            }
        }

        sql.push_str(format!(" FROM {}", table_name).as_str());
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

    fn build_filter_expr(filter: &dyn Filtered) -> String {
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
            let left = Self::build_filter_expr(f1);
            let right = Self::build_filter_expr(f2);
            return format!("({} {} {})", left, op, right);
        }
        let Some(col1) = filter.column_one() else {
            eprintln!("Warning: Simple filter missing column_one, using tautology");
            return "1=1".to_string();
        };
        if let Some(value) = filter.value() {
            match value {
                Value::String(inner) => {
                    let escaped = inner.replace('\'', "''");
                    format!(
                        "{}.{} {} '{}'",
                        col1.0,
                        col1.1,
                        filter.filter_type().to_sql(),
                        escaped
                    )
                }
                _ => {
                    format!(
                        "{}.{} {} {}",
                        col1.0,
                        col1.1,
                        filter.filter_type().to_sql(),
                        value
                    )
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

    pub(crate) fn filter_sql(mut sql: String, filters: Vec<Box<dyn Filtered>>) -> String {
        if filters.is_empty() {
            return sql;
        }

        sql.push_str(" WHERE ");
        let mut parts: Vec<String> = Vec::with_capacity(filters.len());
        for filter in &filters {
            parts.push(Self::build_filter_expr(filter.as_ref()));
        }
        sql.push_str(&parts.join(" AND "));

        sql
    }
}
