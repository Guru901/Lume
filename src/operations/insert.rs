#![warn(missing_docs)]

//! # Insert Operation
//!
//! This module provides the [`Insert`] struct for type-safe insertion of records
//! into a MySQL database using a schema definition. It supports optional
//! returning of inserted rows and handles value binding for various SQL types.

use crate::database::error::DatabaseError;
use crate::helpers::{StartingSql, bind_column_value, get_starting_sql, quote_identifier};
use crate::row::Row;
use crate::schema::{ColumnInfo, Schema, Select, Value};

#[cfg(feature = "mysql")]
use sqlx::MySqlPool;

#[cfg(feature = "postgres")]
use sqlx::PgPool;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Select columns that should be included in an INSERT statement based on provided values.
///
/// Omits columns that have defaults or are auto-incremented when their value is absent or Null.
fn select_insertable_columns(
    all_columns: Vec<ColumnInfo>,
    values: &HashMap<String, Value>,
) -> Vec<ColumnInfo> {
    all_columns
        .into_iter()
        .filter(|col| match values.get(col.name) {
            None => !(col.has_default || col.auto_increment),
            Some(Value::Null) => !(col.has_default || col.auto_increment),
            _ => true,
        })
        .collect()
}

/// A type-safe insert operation for a given schema type.
///
/// The [`Insert`] struct allows you to insert a record of type `T` (which must
/// implement [`Schema`] and [`Debug`]) into the corresponding database table.
/// It builds the SQL statement and binds values according to the schema.
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
/// async fn main() {
///     let db = Database::connect("mysql://...").await.unwrap();
///     db.insert(User { id: 1, name: "guru".to_string() })
///         .execute()
///         .await
///         .unwrap();
/// }
/// ```
pub struct Insert<T> {
    /// The data to be inserted.
    data: T,

    #[cfg(feature = "mysql")]
    /// The database connection pool.
    conn: Arc<MySqlPool>,

    #[cfg(feature = "postgres")]
    /// The database connection pool.
    conn: Arc<PgPool>,

    /// Whether to return the inserted row(s).
    returning: Vec<&'static str>,
}

impl<T: Schema + Debug> Insert<T> {
    #[cfg(feature = "mysql")]
    /// Creates a new [`Insert`] operation for the given data and connection.
    ///
    /// # Arguments
    ///
    /// * `data` - The record to insert.
    /// * `conn` - The database connection pool.
    ///
    /// # Returns
    ///
    /// An [`Insert`] instance ready for execution.
    pub fn new(data: T, conn: Arc<MySqlPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    #[cfg(feature = "postgres")]
    /// Creates a new [`Insert`] operation for the given data and connection.
    ///
    /// # Arguments
    ///
    /// * `data` - The record to insert.
    /// * `conn` - The database connection pool.
    ///
    /// # Returns
    ///
    /// An [`Insert`] instance ready for execution.
    pub fn new(data: T, conn: Arc<PgPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    /// Configures the insert to return the inserted row(s).
    ///
    /// # Returns
    ///
    /// The [`Insert`] instance with returning enabled.
    pub fn returning<S: Select + Debug>(mut self, select: S) -> Self {
        self.returning = select.get_selected();
        self
    }

    /// Executes the insert operation asynchronously.
    ///
    /// This method builds the SQL `INSERT` statement, binds all values
    /// according to the schema, and executes the query.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the insert was successful.
    /// * `Err(DatabaseError)` if an error occurred.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use lume::database::Database;
    /// # use lume::define_schema;
    /// # use lume::schema::Schema;
    /// # use lume::schema::ColumnInfo;
    /// # define_schema! {
    /// #     User {
    /// #         id: i32 [primary_key()],
    /// #         name: String [not_null()],
    /// #     }
    /// # }
    /// # #[tokio::main]
    /// # async fn main() {
    /// let db = Database::connect("mysql://...").await.unwrap();
    /// db.insert(User { id: 1, name: "guru".to_string() })
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn execute(self) -> Result<Option<Vec<Row<T>>>, DatabaseError> {
        let mut conn = self.conn.acquire().await?;

        let values = self.data.values();
        let all_columns = T::get_all_columns();

        // Select columns to include: omit columns with defaults/auto_increment when value is None/Null
        let selected: Vec<ColumnInfo> = select_insertable_columns(all_columns, &values);

        let sql = get_starting_sql(StartingSql::Insert, T::table_name());
        let sql = Self::insert_sql(sql, selected.clone());
        let mut query = sqlx::query(&sql);

        for col in selected.iter() {
            let value = values.get(col.name);
            query = bind_column_value(query, col, value);
        }

        // For PostgreSQL with RETURNING, we need to add RETURNING clause to the INSERT
        #[cfg(feature = "postgres")]
        if !self.returning.is_empty() {
            use crate::helpers::returning_sql;

            let sql = get_starting_sql(StartingSql::Insert, T::table_name());
            let sql = Self::insert_sql(sql, selected.clone());
            let sql = returning_sql(sql, &self.returning);
            let mut query = sqlx::query(&sql);

            for col in selected.iter() {
                let value = values.get(col.name);
                query = bind_column_value(query, col, value);
            }

            let rows = query.fetch_all(&mut *conn).await?;
            let rows = Row::<T>::from_postgres_row(rows, None);
            return Ok(Some(rows));
        }

        let _result = query.execute(&mut *conn).await?;

        if self.returning.is_empty() {
            return Ok(None);
        }

        // For MySQL, build SELECT ... WHERE id = ? using either provided id or last_insert_id
        #[cfg(feature = "mysql")]
        {
            use crate::helpers::returning_sql;

            let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
            let mut select_sql = returning_sql(select_sql, &self.returning);
            select_sql.push_str(format!(" FROM {} WHERE id = ?;", T::table_name()).as_str());

            let mut conn = self.conn.acquire().await?;

            let mut query = sqlx::query(&select_sql);

            query = query.bind(_result.last_insert_id());

            let rows = query.fetch_all(&mut *conn).await?;
            let rows = Row::<T>::from_mysql_row(rows, None);
            Ok(Some(rows))
        }

        #[cfg(feature = "postgres")]
        {
            // This should not be reached as we handle RETURNING above
            Ok(None)
        }
    }

    /// Builds a parameterized INSERT SQL statement, with identifier quoting for MySQL/Postgres and parameter style for both backends.
    pub(crate) fn insert_sql(mut sql: String, columns: Vec<ColumnInfo>) -> String {
        // Quote identifiers for portability (uses quote_identifier from lib.rs)
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&quote_identifier(&col.name));
        }
        sql.push_str(") VALUES (");

        #[cfg(feature = "postgres")]
        {
            // Use $1, $2, $3... for Postgres
            for (i, _col) in columns.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push_str(&format!("${}", i + 1));
            }
        }
        #[cfg(feature = "mysql")]
        {
            // Use ? for MySQL
            for (i, _col) in columns.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push_str("?");
            }
        }
        // fallback (support at least something for no features)
        #[cfg(all(not(feature = "postgres"), not(feature = "mysql")))]
        {
            for (i, _col) in columns.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                sql.push_str("?");
            }
        }

        sql.push_str(")");

        sql
    }
}

/// A type-safe insert operation for inserting multiple records of a given schema type.
///
/// Executes one INSERT per record for simplicity and correctness. This can be
/// optimized later to a multi-row VALUES statement if needed.
pub struct InsertMany<T: Schema + Debug> {
    /// The list of records to be inserted.
    data: Vec<T>,

    #[cfg(feature = "mysql")]
    /// The database connection pool.
    conn: Arc<MySqlPool>,

    #[cfg(feature = "postgres")]
    /// The database connection pool.
    conn: Arc<PgPool>,

    /// Whether to return the inserted rows.
    returning: Vec<&'static str>,
}

impl<T: Schema + Debug> InsertMany<T> {
    #[cfg(feature = "mysql")]
    /// Creates a new [`InsertMany`] operation for the given records and connection.
    pub fn new(data: Vec<T>, conn: Arc<MySqlPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    #[cfg(feature = "postgres")]
    /// Creates a new [`InsertMany`] operation for the given records and connection.
    pub fn new(data: Vec<T>, conn: Arc<PgPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    /// Configures the insert to return the inserted row(s).
    pub fn returning<S: Select + Debug>(mut self, select: S) -> Self {
        self.returning = select.get_selected();
        self
    }

    /// Executes the insert operation for all records asynchronously.
    pub async fn execute(self) -> Result<Option<Vec<Row<T>>>, DatabaseError> {
        let mut conn = self.conn.acquire().await?;
        let mut final_rows = Vec::new();
        let mut inserted_ids: Vec<u64> = Vec::new();

        for record in &self.data {
            let values = record.values();
            let all_columns = T::get_all_columns();
            let selected: Vec<ColumnInfo> = select_insertable_columns(all_columns, &values);

            let sql = get_starting_sql(StartingSql::Insert, T::table_name());
            let sql = Insert::<T>::insert_sql(sql, selected.clone());
            let mut query = sqlx::query(&sql);

            for col in selected.iter() {
                let value = values.get(col.name);
                query = bind_column_value(query, col, value);
            }

            #[cfg(feature = "mysql")]
            {
                let result = query.execute(&mut *conn).await?;

                // Capture id: prefer provided id, else last_insert_id
                if let Some(id_val) = values.get("id") {
                    match id_val {
                        Value::Array(_) => inserted_ids.push(result.last_insert_id()),
                        Value::Int64(v) => inserted_ids.push(*v as u64),
                        Value::Int32(v) => inserted_ids.push(*v as u64),
                        Value::Int16(v) => inserted_ids.push(*v as u64),
                        Value::Int8(v) => inserted_ids.push(*v as u64),
                        Value::UInt64(v) => inserted_ids.push(*v),
                        Value::UInt32(v) => inserted_ids.push(*v as u64),
                        Value::UInt16(v) => inserted_ids.push(*v as u64),
                        Value::UInt8(v) => inserted_ids.push(*v as u64),
                        Value::String(_)
                        | Value::Float32(_)
                        | Value::Float64(_)
                        | Value::Bool(_)
                        | Value::Between(_, _)
                        | Value::Null => inserted_ids.push(result.last_insert_id()),
                    }
                } else {
                    inserted_ids.push(result.last_insert_id());
                }
            }

            #[cfg(feature = "postgres")]
            {
                // For PostgreSQL, if returning is requested, we need to use RETURNING clause
                if !self.returning.is_empty() {
                    use crate::helpers::returning_sql;

                    let sql = get_starting_sql(StartingSql::Insert, T::table_name());
                    let sql = Insert::<T>::insert_sql(sql, selected.clone());
                    let sql = returning_sql(sql, &self.returning);
                    let mut query = sqlx::query(&sql);

                    for col in selected.iter() {
                        let value = values.get(col.name);
                        query = bind_column_value(query, col, value);
                    }

                    let rows = query.fetch_all(&mut *conn).await?;
                    let rows = Row::<T>::from_postgres_row(rows, None);
                    final_rows.extend(rows);
                } else {
                    // Execute without returning
                    query.execute(&mut *conn).await?;

                    // Capture id: prefer provided id
                    if let Some(id_val) = values.get("id") {
                        match id_val {
                            Value::Int64(v) => inserted_ids.push(*v as u64),
                            Value::Int32(v) => inserted_ids.push(*v as u64),
                            Value::Int16(v) => inserted_ids.push(*v as u64),
                            Value::Int8(v) => inserted_ids.push(*v as u64),
                            _ => {
                                // For PostgreSQL without RETURNING, we can't get the id
                                // This is a limitation - user should use returning() to get ids
                            }
                        }
                    }
                }
            }
        }

        #[cfg(feature = "mysql")]
        {
            use crate::helpers::returning_sql;

            if self.returning.is_empty() {
                return Ok(None);
            }

            // Fetch selected columns for all inserted ids
            let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
            let mut select_sql = returning_sql(select_sql, &self.returning);
            select_sql.push_str(format!(" FROM {} WHERE id = ?;", T::table_name()).as_str());

            for id in inserted_ids {
                let q = sqlx::query(&select_sql).bind(id);
                let rows = q.fetch_all(&mut *conn).await?;

                let rows = Row::<T>::from_mysql_row(rows, None);
                final_rows.extend(rows);
            }

            Ok(Some(final_rows))
        }

        #[cfg(feature = "postgres")]
        {
            // For PostgreSQL, if returning was used, rows are already in final_rows
            if !self.returning.is_empty() {
                Ok(Some(final_rows))
            } else if !inserted_ids.is_empty() {
                // Fetch selected columns for all inserted ids

                use crate::helpers::returning_sql;
                let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
                let mut select_sql = returning_sql(select_sql, &self.returning);
                select_sql.push_str(format!(" FROM {} WHERE id = $1;", T::table_name()).as_str());

                for id in inserted_ids {
                    let q = sqlx::query(&select_sql).bind(id as i64);
                    let rows = q.fetch_all(&mut *conn).await?;

                    let rows = Row::<T>::from_postgres_row(rows, None);
                    final_rows.extend(rows);
                }

                Ok(Some(final_rows))
            } else {
                Ok(None)
            }
        }
    }
}
