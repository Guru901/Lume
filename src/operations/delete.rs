use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use sqlx::MySqlPool;

use crate::{
    StartingSql,
    database::DatabaseError,
    filter::Filtered,
    get_starting_sql,
    row::Row,
    schema::{Schema, Value},
};

pub struct Delete<T> {
    table: PhantomData<T>,
    /// List of filters to apply to the query
    filters: Vec<Box<dyn Filtered>>,
    /// Database connection pool
    conn: Arc<MySqlPool>,
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
    pub fn new(conn: Arc<MySqlPool>) -> Self {
        Self {
            table: PhantomData,
            conn,
            filters: Vec::new(),
        }
    }

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
    pub async fn execute(self) -> Result<(), DatabaseError> {
        let sql = get_starting_sql(StartingSql::Delete, T::table_name());

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
                Value::UInt16(u) => query.bind(u),
                Value::UInt32(u) => query.bind(u),
                Value::UInt64(u) => query.bind(u),
                Value::Float32(f) => query.bind(f),
                Value::Float64(f) => query.bind(f),
                Value::Bool(b) => query.bind(b),
                Value::Null => query, // Nulls handled in SQL via IS/IS NOT
            };
        }

        println!("SQL: {sql}");

        return Ok(());

        query
            .execute(&mut *conn)
            .await
            .map_err(DatabaseError::from)?;

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
            parts.push(Self::build_filter_expr(filter.as_ref(), params));
        }
        sql.push_str(&parts.join(" AND "));

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
}
