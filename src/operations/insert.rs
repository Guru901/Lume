#![warn(missing_docs)]

//! # Insert Operation
//!
//! This module provides the [`Insert`] struct for type-safe insertion of records
//! into a MySQL database using a schema definition. It supports optional
//! returning of inserted rows and handles value binding for various SQL types.

use crate::database::error::DatabaseError;
use crate::row::Row;
use crate::schema::{ColumnInfo, Schema, Select, Value};
use crate::{StartingSql, get_starting_sql, returning_sql};

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
            let Some(value) = values.get(col.name) else {
                // If a value is missing, bind NULL using the column's SQL type.
                match col.data_type {
                    "VARCHAR(255)" | "TEXT" => {
                        query = query.bind(None::<&str>);
                    }
                    "INTEGER" => {
                        query = query.bind(None::<i32>);
                    }
                    "BIGINT" => {
                        query = query.bind(None::<i64>);
                    }
                    "FLOAT" => {
                        query = query.bind(None::<f32>);
                    }
                    "DOUBLE" => {
                        query = query.bind(None::<f64>);
                    }
                    "BOOLEAN" => {
                        query = query.bind(None::<bool>);
                    }
                    _ => {
                        query = query.bind(None::<&str>);
                    }
                }
                continue;
            };

            match value {
                Value::Array(_arr) => {
                    // Arrays are not directly insertable; bind NULL using the column's SQL type
                    match col.data_type {
                        "VARCHAR(255)" | "TEXT" => {
                            query = query.bind(None::<&str>);
                        }
                        "TINYINT" => {
                            query = query.bind(None::<i8>);
                        }
                        "SMALLINT" => {
                            query = query.bind(None::<i16>);
                        }
                        "INTEGER" => {
                            query = query.bind(None::<i32>);
                        }
                        "BIGINT" => {
                            query = query.bind(None::<i64>);
                        }
                        #[cfg(feature = "mysql")]
                        "TINYINT UNSIGNED" => {
                            query = query.bind(None::<u8>);
                        }

                        #[cfg(feature = "mysql")]
                        "SMALLINT UNSIGNED" => {
                            query = query.bind(None::<u16>);
                        }

                        #[cfg(feature = "mysql")]
                        "INTEGER UNSIGNED" => {
                            query = query.bind(None::<u32>);
                        }

                        #[cfg(feature = "mysql")]
                        "BIGINT UNSIGNED" => {
                            query = query.bind(None::<u64>);
                        }
                        "FLOAT" => {
                            query = query.bind(None::<f32>);
                        }
                        "DOUBLE" => {
                            query = query.bind(None::<f64>);
                        }
                        "BOOLEAN" => {
                            query = query.bind(None::<bool>);
                        }
                        _ => {
                            query = query.bind(None::<&str>);
                        }
                    }
                }
                Value::Int8(v) => {
                    query = query.bind(*v);
                }
                Value::Int16(v) => {
                    query = query.bind(*v);
                }
                Value::Int32(v) => {
                    query = query.bind(*v);
                }
                Value::Int64(v) => {
                    query = query.bind(*v);
                }

                #[cfg(feature = "mysql")]
                Value::UInt8(v) => {
                    query = query.bind(*v);
                }

                #[cfg(feature = "postgres")]
                Value::UInt16(v) => {
                    query = query.bind(*v as i32);
                }
                #[cfg(feature = "postgres")]
                Value::UInt32(v) => {
                    query = query.bind(*v as i64);
                }
                #[cfg(feature = "postgres")]
                Value::UInt64(v) => {
                    query = query.bind(*v as i64);
                }

                #[cfg(feature = "mysql")]
                Value::UInt16(v) => {
                    query = query.bind(*v);
                }
                #[cfg(feature = "mysql")]
                Value::UInt32(v) => {
                    query = query.bind(*v);
                }
                #[cfg(feature = "mysql")]
                Value::UInt64(v) => {
                    query = query.bind(*v);
                }

                Value::Float32(v) => {
                    query = query.bind(*v);
                }
                Value::Float64(v) => {
                    query = query.bind(*v);
                }
                Value::Bool(v) => {
                    query = query.bind(*v);
                }
                Value::String(v) => {
                    query = query.bind(v.as_str());
                }
                Value::Null => match col.data_type {
                    "VARCHAR(255)" | "TEXT" => {
                        query = query.bind(None::<&str>);
                    }
                    "TINYINT" => {
                        query = query.bind(None::<i8>);
                    }
                    "SMALLINT" => {
                        query = query.bind(None::<i16>);
                    }
                    "INTEGER" => {
                        query = query.bind(None::<i32>);
                    }
                    "BIGINT" => {
                        query = query.bind(None::<i64>);
                    }
                    #[cfg(feature = "mysql")]
                    "TINYINT UNSIGNED" => {
                        query = query.bind(None::<u8>);
                    }
                    #[cfg(feature = "mysql")]
                    "SMALLINT UNSIGNED" => {
                        query = query.bind(None::<u16>);
                    }
                    #[cfg(feature = "mysql")]
                    "INTEGER UNSIGNED" => {
                        query = query.bind(None::<u32>);
                    }
                    #[cfg(feature = "mysql")]
                    "BIGINT UNSIGNED" => {
                        query = query.bind(None::<u64>);
                    }
                    "FLOAT" => {
                        query = query.bind(None::<f32>);
                    }
                    "DOUBLE" => {
                        query = query.bind(None::<f64>);
                    }
                    "BOOLEAN" => {
                        query = query.bind(None::<bool>);
                    }
                    _ => {
                        query = query.bind(None::<&str>);
                    }
                },
                #[cfg(feature = "mysql")]
                Value::UInt8(_) | Value::UInt16(_) | Value::UInt32(_) | Value::UInt64(_) => {
                    // These are already handled above in the match arms
                    unreachable!("UInt types should be handled in specific match arms")
                }
                Value::Between(min, max) => {
                    query = match (**min).clone() {
                        Value::String(s) => query.bind(s),
                        Value::Int8(i) => query.bind(i),
                        Value::Int16(i) => query.bind(i),
                        Value::Int32(i) => query.bind(i),
                        Value::Int64(i) => query.bind(i),

                        #[cfg(feature = "mysql")]
                        Value::UInt8(v) => query.bind(v),

                        #[cfg(feature = "postgres")]
                        Value::UInt16(v) => query.bind(v as i32),
                        #[cfg(feature = "postgres")]
                        Value::UInt32(v) => query.bind(v as i64),
                        #[cfg(feature = "postgres")]
                        Value::UInt64(v) => query.bind(v as i64),

                        #[cfg(feature = "mysql")]
                        Value::UInt16(v) => query.bind(v),
                        #[cfg(feature = "mysql")]
                        Value::UInt32(v) => query.bind(v),
                        #[cfg(feature = "mysql")]
                        Value::UInt64(v) => query.bind(v),

                        Value::Float32(f) => query.bind(f),
                        Value::Float64(f) => query.bind(f),
                        Value::Bool(b) => query.bind(b),
                        Value::Array(_arr) => {
                            eprintln!(
                                "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                            );
                            query
                        }
                        Value::Between(_, _) => {
                            eprintln!(
                                "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                            );
                            query
                        }
                        Value::Null => query,
                    };
                    query = match (**max).clone() {
                        Value::String(s) => query.bind(s),
                        Value::Int8(i) => query.bind(i),
                        Value::Int16(i) => query.bind(i),
                        Value::Int32(i) => query.bind(i),
                        Value::Int64(i) => query.bind(i),
                        #[cfg(feature = "mysql")]
                        Value::UInt8(u) => query.bind(u),
                        Value::UInt16(u) => query.bind(u as i32),
                        Value::UInt32(u) => query.bind(u as i64),
                        Value::UInt64(u) => query.bind(u as i64),
                        Value::Float32(f) => query.bind(f),
                        Value::Float64(f) => query.bind(f),
                        Value::Bool(b) => query.bind(b),
                        Value::Array(_arr) => {
                            eprintln!(
                                "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                            );
                            query
                        }
                        Value::Between(_, _) => {
                            eprintln!(
                                "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                            );
                            query
                        }
                        Value::Null => query,
                    };
                }
            }
        }

        // For PostgreSQL with RETURNING, we need to add RETURNING clause to the INSERT
        #[cfg(feature = "postgres")]
        if !self.returning.is_empty() {
            let sql = get_starting_sql(StartingSql::Insert, T::table_name());
            let sql = Self::insert_sql(sql, selected.clone());
            let sql = returning_sql(sql, &self.returning);
            let mut query = sqlx::query(&sql);

            for col in selected.iter() {
                let Some(value) = values.get(col.name) else {
                    match col.data_type {
                        "VARCHAR(255)" | "TEXT" => {
                            query = query.bind(None::<&str>);
                        }
                        "TINYINT" => {
                            query = query.bind(None::<i8>);
                        }
                        "SMALLINT" => {
                            query = query.bind(None::<i16>);
                        }
                        "INTEGER" => {
                            query = query.bind(None::<i32>);
                        }
                        "BIGINT" => {
                            query = query.bind(None::<i64>);
                        }
                        "FLOAT" => {
                            query = query.bind(None::<f32>);
                        }
                        "DOUBLE" => {
                            query = query.bind(None::<f64>);
                        }
                        "BOOLEAN" => {
                            query = query.bind(None::<bool>);
                        }
                        _ => {
                            query = query.bind(None::<&str>);
                        }
                    }
                    continue;
                };

                match value {
                    Value::Array(_arr) => match col.data_type {
                        "VARCHAR(255)" | "TEXT" => {
                            query = query.bind(None::<&str>);
                        }
                        "TINYINT" => {
                            query = query.bind(None::<i8>);
                        }
                        "SMALLINT" => {
                            query = query.bind(None::<i16>);
                        }
                        "INTEGER" => {
                            query = query.bind(None::<i32>);
                        }
                        "BIGINT" => {
                            query = query.bind(None::<i64>);
                        }
                        "FLOAT" => {
                            query = query.bind(None::<f32>);
                        }
                        "DOUBLE" => {
                            query = query.bind(None::<f64>);
                        }
                        "BOOLEAN" => {
                            query = query.bind(None::<bool>);
                        }
                        _ => {
                            query = query.bind(None::<&str>);
                        }
                    },
                    Value::Int8(v) => {
                        query = query.bind(*v);
                    }
                    Value::Int16(v) => {
                        query = query.bind(*v);
                    }
                    Value::Int32(v) => {
                        query = query.bind(*v);
                    }
                    Value::Int64(v) => {
                        query = query.bind(*v);
                    }
                    Value::Float32(v) => {
                        query = query.bind(*v);
                    }
                    Value::Float64(v) => {
                        query = query.bind(*v);
                    }
                    Value::Bool(v) => {
                        query = query.bind(*v);
                    }
                    Value::String(v) => {
                        query = query.bind(v.as_str());
                    }
                    Value::Null => match col.data_type {
                        "VARCHAR(255)" | "TEXT" => {
                            query = query.bind(None::<&str>);
                        }
                        "TINYINT" => {
                            query = query.bind(None::<i8>);
                        }
                        "SMALLINT" => {
                            query = query.bind(None::<i16>);
                        }
                        "INTEGER" => {
                            query = query.bind(None::<i32>);
                        }
                        "BIGINT" => {
                            query = query.bind(None::<i64>);
                        }
                        "FLOAT" => {
                            query = query.bind(None::<f32>);
                        }
                        "DOUBLE" => {
                            query = query.bind(None::<f64>);
                        }
                        "BOOLEAN" => {
                            query = query.bind(None::<bool>);
                        }
                        _ => {
                            query = query.bind(None::<&str>);
                        }
                    },
                    Value::Between(min, max) => {
                        query = match (**min).clone() {
                            Value::String(s) => query.bind(s),
                            Value::Int8(i) => query.bind(i),
                            Value::Int16(i) => query.bind(i),
                            Value::Int32(i) => query.bind(i),
                            Value::Int64(i) => query.bind(i),
                            Value::UInt16(u) => query.bind(u as i32),
                            Value::UInt32(u) => query.bind(u as i64),
                            Value::UInt64(u) => query.bind(u as i64),
                            Value::Float32(f) => query.bind(f),
                            Value::Float64(f) => query.bind(f),
                            Value::Bool(b) => query.bind(b),
                            Value::Array(_arr) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                                );
                                query
                            }
                            Value::Between(_, _) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                                );
                                query
                            }
                            Value::Null => query,
                        };
                        query = match (**max).clone() {
                            Value::String(s) => query.bind(s),
                            Value::Int8(i) => query.bind(i),
                            Value::Int16(i) => query.bind(i),
                            Value::Int32(i) => query.bind(i),
                            Value::Int64(i) => query.bind(i),
                            Value::Float32(f) => query.bind(f),
                            Value::Float64(f) => query.bind(f),
                            Value::Bool(b) => query.bind(b),
                            Value::Array(_arr) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                                );
                                query
                            }
                            Value::Between(_, _) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                                );
                                query
                            }
                            Value::Null => query,
                            Value::UInt16(v) => query.bind(v as i32),
                            Value::UInt32(v) => query.bind(v as i64),
                            Value::UInt64(v) => query.bind(v as i64),
                        };
                        query = match (**max).clone() {
                            Value::String(s) => query.bind(s),
                            Value::Int8(i) => query.bind(i),
                            Value::Int16(i) => query.bind(i),
                            Value::Int32(i) => query.bind(i),
                            Value::Int64(i) => query.bind(i),
                            Value::Float32(f) => query.bind(f),
                            Value::Float64(f) => query.bind(f),
                            Value::Bool(b) => query.bind(b),
                            Value::Array(_arr) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                                );
                                query
                            }
                            Value::Between(_, _) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                                );
                                query
                            }
                            Value::Null => query,
                            Value::UInt16(v) => query.bind(v as i32),
                            Value::UInt32(v) => query.bind(v as i64),
                            Value::UInt64(v) => query.bind(v as i64),
                        };
                    }
                    Value::UInt16(v) => {
                        query = query.bind(*v as i32);
                    }
                    Value::UInt32(v) => {
                        query = query.bind(*v as i64);
                    }
                    Value::UInt64(v) => {
                        query = query.bind(*v as i64);
                    }
                }
            }

            let rows = query.fetch_all(&mut *conn).await?;
            let rows = Row::<T>::from_postgres_row(rows, None);
            return Ok(Some(rows));
        }

        let result = query.execute(&mut *conn).await?;

        if self.returning.is_empty() {
            return Ok(None);
        }

        // For MySQL, build SELECT ... WHERE id = ? using either provided id or last_insert_id
        #[cfg(feature = "mysql")]
        {
            let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
            let mut select_sql = returning_sql(select_sql, &self.returning);
            select_sql.push_str(format!(" FROM {} WHERE id = ?;", T::table_name()).as_str());

            let mut conn = self.conn.acquire().await?;

            let mut query = sqlx::query(&select_sql);

            query = query.bind(result.last_insert_id());

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

    pub(crate) fn insert_sql(mut sql: String, columns: Vec<ColumnInfo>) -> String {
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&col.name);
        }
        sql.push_str(") VALUES (");

        for (i, _col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str("?");
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
                let Some(value) = values.get(col.name) else {
                    // If a value is missing, bind NULL using the column's SQL type.
                    match col.data_type {
                        "VARCHAR(255)" | "TEXT" => {
                            query = query.bind(None::<&str>);
                        }
                        "INTEGER" => {
                            query = query.bind(None::<i32>);
                        }
                        "BIGINT" => {
                            query = query.bind(None::<i64>);
                        }
                        "FLOAT" => {
                            query = query.bind(None::<f32>);
                        }
                        "DOUBLE" => {
                            query = query.bind(None::<f64>);
                        }
                        "BOOLEAN" => {
                            query = query.bind(None::<bool>);
                        }
                        _ => {
                            query = query.bind(None::<&str>);
                        }
                    }
                    continue;
                };

                match value {
                    Value::Array(_arr) => {
                        // Arrays are not directly insertable; bind NULL using the column's SQL type
                        match col.data_type {
                            "VARCHAR(255)" | "TEXT" => {
                                query = query.bind(None::<&str>);
                            }
                            "TINYINT" => {
                                query = query.bind(None::<i8>);
                            }
                            "SMALLINT" => {
                                query = query.bind(None::<i16>);
                            }
                            "INTEGER" => {
                                query = query.bind(None::<i32>);
                            }
                            "BIGINT" => {
                                query = query.bind(None::<i64>);
                            }
                            #[cfg(feature = "mysql")]
                            "TINYINT UNSIGNED" => {
                                query = query.bind(None::<u8>);
                            }
                            #[cfg(feature = "mysql")]
                            "SMALLINT UNSIGNED" => {
                                query = query.bind(None::<u16>);
                            }
                            #[cfg(feature = "mysql")]
                            "INTEGER UNSIGNED" => {
                                query = query.bind(None::<u32>);
                            }
                            #[cfg(feature = "mysql")]
                            "BIGINT UNSIGNED" => {
                                query = query.bind(None::<u64>);
                            }
                            "FLOAT" => {
                                query = query.bind(None::<f32>);
                            }
                            "DOUBLE" => {
                                query = query.bind(None::<f64>);
                            }
                            "BOOLEAN" => {
                                query = query.bind(None::<bool>);
                            }
                            _ => {
                                query = query.bind(None::<&str>);
                            }
                        }
                    }
                    Value::Int8(v) => {
                        query = query.bind(*v);
                    }
                    Value::Int16(v) => {
                        query = query.bind(*v);
                    }
                    Value::Int32(v) => {
                        query = query.bind(*v);
                    }
                    Value::Int64(v) => {
                        query = query.bind(*v);
                    }

                    #[cfg(feature = "mysql")]
                    Value::UInt8(v) => {
                        query = query.bind(*v);
                    }

                    #[cfg(feature = "postgres")]
                    Value::UInt16(v) => {
                        query = query.bind(*v as i32);
                    }
                    #[cfg(feature = "postgres")]
                    Value::UInt32(v) => {
                        query = query.bind(*v as i64);
                    }
                    #[cfg(feature = "postgres")]
                    Value::UInt64(v) => {
                        query = query.bind(*v as i64);
                    }

                    #[cfg(feature = "mysql")]
                    Value::UInt16(v) => {
                        query = query.bind(*v);
                    }
                    #[cfg(feature = "mysql")]
                    Value::UInt32(v) => {
                        query = query.bind(*v);
                    }
                    #[cfg(feature = "mysql")]
                    Value::UInt64(v) => {
                        query = query.bind(*v);
                    }

                    Value::Float32(v) => {
                        query = query.bind(*v);
                    }
                    Value::Float64(v) => {
                        query = query.bind(*v);
                    }
                    Value::Bool(v) => {
                        query = query.bind(*v);
                    }
                    Value::String(v) => {
                        query = query.bind(v.as_str());
                    }
                    Value::Null => match col.data_type {
                        "VARCHAR(255)" | "TEXT" => {
                            query = query.bind(None::<&str>);
                        }
                        "TINYINT" => {
                            query = query.bind(None::<i8>);
                        }
                        "SMALLINT" => {
                            query = query.bind(None::<i16>);
                        }
                        "INTEGER" => {
                            query = query.bind(None::<i32>);
                        }
                        "BIGINT" => {
                            query = query.bind(None::<i64>);
                        }
                        #[cfg(feature = "mysql")]
                        "TINYINT UNSIGNED" => {
                            query = query.bind(None::<u8>);
                        }
                        #[cfg(feature = "mysql")]
                        "SMALLINT UNSIGNED" => {
                            query = query.bind(None::<u16>);
                        }
                        #[cfg(feature = "mysql")]
                        "INTEGER UNSIGNED" => {
                            query = query.bind(None::<u32>);
                        }
                        #[cfg(feature = "mysql")]
                        "BIGINT UNSIGNED" => {
                            query = query.bind(None::<u64>);
                        }
                        "FLOAT" => {
                            query = query.bind(None::<f32>);
                        }
                        "DOUBLE" => {
                            query = query.bind(None::<f64>);
                        }
                        "BOOLEAN" => {
                            query = query.bind(None::<bool>);
                        }
                        _ => {
                            query = query.bind(None::<&str>);
                        }
                    },
                    Value::Between(min, max) => {
                        query = match (**min).clone() {
                            Value::String(s) => query.bind(s),
                            Value::Int8(i) => query.bind(i),
                            Value::Int16(i) => query.bind(i),
                            Value::Int32(i) => query.bind(i),
                            Value::Int64(i) => query.bind(i),

                            #[cfg(feature = "mysql")]
                            Value::UInt8(u) => query.bind(u),

                            #[cfg(feature = "postgres")]
                            Value::UInt16(u) => query.bind(u as i32),
                            #[cfg(feature = "postgres")]
                            Value::UInt32(u) => query.bind(u as i64),
                            #[cfg(feature = "postgres")]
                            Value::UInt64(u) => query.bind(u as i64),

                            #[cfg(feature = "mysql")]
                            Value::UInt16(u) => query.bind(u),
                            #[cfg(feature = "mysql")]
                            Value::UInt32(u) => query.bind(u),
                            #[cfg(feature = "mysql")]
                            Value::UInt64(u) => query.bind(u),

                            Value::Float32(f) => query.bind(f),
                            Value::Float64(f) => query.bind(f),
                            Value::Bool(b) => query.bind(b),
                            Value::Array(_arr) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                                );
                                query
                            }
                            Value::Between(_, _) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                                );
                                query
                            }
                            Value::Null => query,
                        };
                        query = match (**max).clone() {
                            Value::String(s) => query.bind(s),
                            Value::Int8(i) => query.bind(i),
                            Value::Int16(i) => query.bind(i),
                            Value::Int32(i) => query.bind(i),
                            Value::Int64(i) => query.bind(i),
                            #[cfg(feature = "mysql")]
                            Value::UInt8(u) => query.bind(u),

                            #[cfg(feature = "postgres")]
                            Value::UInt16(u) => query.bind(u as i32),
                            #[cfg(feature = "postgres")]
                            Value::UInt32(u) => query.bind(u as i64),
                            #[cfg(feature = "postgres")]
                            Value::UInt64(u) => query.bind(u as i64),

                            #[cfg(feature = "mysql")]
                            Value::UInt16(u) => query.bind(u),
                            #[cfg(feature = "mysql")]
                            Value::UInt32(u) => query.bind(u),
                            #[cfg(feature = "mysql")]
                            Value::UInt64(u) => query.bind(u),

                            Value::Float32(f) => query.bind(f),
                            Value::Float64(f) => query.bind(f),
                            Value::Bool(b) => query.bind(b),
                            Value::Array(_arr) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                                );
                                query
                            }
                            Value::Between(_, _) => {
                                eprintln!(
                                    "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                                );
                                query
                            }
                            Value::Null => query,
                        };
                    }
                }
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
                    let sql = get_starting_sql(StartingSql::Insert, T::table_name());
                    let sql = Insert::<T>::insert_sql(sql, selected.clone());
                    let sql = returning_sql(sql, &self.returning);
                    let mut query = sqlx::query(&sql);

                    for col in selected.iter() {
                        let Some(value) = values.get(col.name) else {
                            match col.data_type {
                                "VARCHAR(255)" | "TEXT" => {
                                    query = query.bind(None::<&str>);
                                }
                                "TINYINT" => {
                                    query = query.bind(None::<i8>);
                                }
                                "SMALLINT" => {
                                    query = query.bind(None::<i16>);
                                }
                                "INTEGER" => {
                                    query = query.bind(None::<i32>);
                                }
                                "BIGINT" => {
                                    query = query.bind(None::<i64>);
                                }
                                "FLOAT" => {
                                    query = query.bind(None::<f32>);
                                }
                                "DOUBLE" => {
                                    query = query.bind(None::<f64>);
                                }
                                "BOOLEAN" => {
                                    query = query.bind(None::<bool>);
                                }
                                _ => {
                                    query = query.bind(None::<&str>);
                                }
                            }
                            continue;
                        };

                        match value {
                            Value::Array(_arr) => match col.data_type {
                                "VARCHAR(255)" | "TEXT" => {
                                    query = query.bind(None::<&str>);
                                }
                                "TINYINT" => {
                                    query = query.bind(None::<i8>);
                                }
                                "SMALLINT" => {
                                    query = query.bind(None::<i16>);
                                }
                                "INTEGER" => {
                                    query = query.bind(None::<i32>);
                                }
                                "BIGINT" => {
                                    query = query.bind(None::<i64>);
                                }
                                "FLOAT" => {
                                    query = query.bind(None::<f32>);
                                }
                                "DOUBLE" => {
                                    query = query.bind(None::<f64>);
                                }
                                "BOOLEAN" => {
                                    query = query.bind(None::<bool>);
                                }
                                _ => {
                                    query = query.bind(None::<&str>);
                                }
                            },
                            Value::Int8(v) => {
                                query = query.bind(*v);
                            }
                            Value::Int16(v) => {
                                query = query.bind(*v);
                            }
                            Value::Int32(v) => {
                                query = query.bind(*v);
                            }
                            Value::Int64(v) => {
                                query = query.bind(*v);
                            }
                            Value::UInt16(v) => {
                                query = query.bind(*v as i32);
                            }
                            Value::UInt32(v) => {
                                query = query.bind(*v as i64);
                            }
                            Value::UInt64(v) => {
                                query = query.bind(*v as i64);
                            }
                            Value::Float32(v) => {
                                query = query.bind(*v);
                            }
                            Value::Float64(v) => {
                                query = query.bind(*v);
                            }
                            Value::Bool(v) => {
                                query = query.bind(*v);
                            }
                            Value::String(v) => {
                                query = query.bind(v.as_str());
                            }
                            Value::Null => match col.data_type {
                                "VARCHAR(255)" | "TEXT" => {
                                    query = query.bind(None::<&str>);
                                }
                                "TINYINT" => {
                                    query = query.bind(None::<i8>);
                                }
                                "SMALLINT" => {
                                    query = query.bind(None::<i16>);
                                }
                                "INTEGER" => {
                                    query = query.bind(None::<i32>);
                                }
                                "BIGINT" => {
                                    query = query.bind(None::<i64>);
                                }
                                "FLOAT" => {
                                    query = query.bind(None::<f32>);
                                }
                                "DOUBLE" => {
                                    query = query.bind(None::<f64>);
                                }
                                "BOOLEAN" => {
                                    query = query.bind(None::<bool>);
                                }
                                _ => {
                                    query = query.bind(None::<&str>);
                                }
                            },
                            Value::Between(min, max) => {
                                query = match (**min).clone() {
                                    Value::String(s) => query.bind(s),
                                    Value::Int8(i) => query.bind(i),
                                    Value::Int16(i) => query.bind(i),
                                    Value::Int32(i) => query.bind(i),
                                    Value::Int64(i) => query.bind(i),
                                    Value::UInt16(u) => query.bind(u as i32),
                                    Value::UInt32(u) => query.bind(u as i64),
                                    Value::UInt64(u) => query.bind(u as i64),
                                    Value::Float32(f) => query.bind(f),
                                    Value::Float64(f) => query.bind(f),
                                    Value::Bool(b) => query.bind(b),
                                    Value::Array(_arr) => {
                                        eprintln!(
                                            "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                                        );
                                        query
                                    }
                                    Value::Between(_, _) => {
                                        eprintln!(
                                            "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                                        );
                                        query
                                    }
                                    Value::Null => query,
                                };
                                query = match (**max).clone() {
                                    Value::String(s) => query.bind(s),
                                    Value::Int8(i) => query.bind(i),
                                    Value::Int16(i) => query.bind(i),
                                    Value::Int32(i) => query.bind(i),
                                    Value::Int64(i) => query.bind(i),
                                    Value::UInt16(u) => query.bind(u as i32),
                                    Value::UInt32(u) => query.bind(u as i64),
                                    Value::UInt64(u) => query.bind(u as i64),
                                    Value::Float32(f) => query.bind(f),
                                    Value::Float64(f) => query.bind(f),
                                    Value::Bool(b) => query.bind(b),
                                    Value::Array(_arr) => {
                                        eprintln!(
                                            "Warning: Attempted to bind Value::Array, which is not supported. Skipping."
                                        );
                                        query
                                    }
                                    Value::Between(_, _) => {
                                        eprintln!(
                                            "Warning: Attempted to bind Value::Between directly, which is not supported. Use the individual min/max values instead."
                                        );
                                        query
                                    }
                                    Value::Null => query,
                                };
                            }
                        }
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
