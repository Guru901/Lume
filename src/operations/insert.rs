#![warn(missing_docs)]

//! # Insert Operation
//!
//! This module provides the [`Insert`] struct for type-safe insertion of records
//! into a MySQL database using a schema definition. It supports optional
//! returning of inserted rows and handles value binding for various SQL types.

use crate::database::DatabaseError;
use crate::row::Row;
use crate::schema::{ColumnInfo, Schema, Select, Value};
use crate::{StartingSql, get_starting_sql, returning_sql};
use sqlx::MySqlPool;
use std::fmt::Debug;
use std::sync::Arc;

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
    /// The database connection pool.
    conn: Arc<MySqlPool>,
    /// Whether to return the inserted row(s).
    returning: Vec<&'static str>,
}

impl<T: Schema + Debug> Insert<T> {
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
        let sql = get_starting_sql(StartingSql::Insert, T::table_name());
        let sql = Self::insert_sql(sql, T::get_all_columns());

        let mut conn = self.conn.acquire().await?;

        let values = self.data.values();
        let columns = T::get_all_columns();
        let mut query = sqlx::query(&sql);

        for col in columns.iter() {
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
                        "TINYINT UNSIGNED" => {
                            query = query.bind(None::<u8>);
                        }
                        "SMALLINT UNSIGNED" => {
                            query = query.bind(None::<u16>);
                        }
                        "INTEGER UNSIGNED" => {
                            query = query.bind(None::<u32>);
                        }
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
                Value::UInt8(v) => {
                    query = query.bind(*v);
                }
                Value::UInt16(v) => {
                    query = query.bind(*v);
                }
                Value::UInt32(v) => {
                    query = query.bind(*v);
                }
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
                    "TINYINT UNSIGNED" => {
                        query = query.bind(None::<u8>);
                    }
                    "SMALLINT UNSIGNED" => {
                        query = query.bind(None::<u16>);
                    }
                    "INTEGER UNSIGNED" => {
                        query = query.bind(None::<u32>);
                    }
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
            }
        }

        let result = query.execute(&mut *conn).await?;

        if self.returning.is_empty() {
            return Ok(None);
        }

        // Build SELECT ... WHERE id = ? using either provided id or last_insert_id
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
    /// The database connection pool.
    conn: Arc<MySqlPool>,
    /// Whether to return the inserted rows.
    returning: Vec<&'static str>,
}

impl<T: Schema + Debug> InsertMany<T> {
    /// Creates a new [`InsertMany`] operation for the given records and connection.
    pub fn new(data: Vec<T>, conn: Arc<MySqlPool>) -> Self {
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
        let sql = get_starting_sql(StartingSql::Insert, T::table_name());
        let sql = Insert::<T>::insert_sql(sql, T::get_all_columns());

        let mut conn = self.conn.acquire().await?;
        let mut final_rows = Vec::new();
        let mut inserted_ids: Vec<u64> = Vec::new();

        for record in &self.data {
            let values = record.values();
            let columns = T::get_all_columns();
            let mut query = sqlx::query(&sql);

            for col in columns.iter() {
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
                            "TINYINT UNSIGNED" => {
                                query = query.bind(None::<u8>);
                            }
                            "SMALLINT UNSIGNED" => {
                                query = query.bind(None::<u16>);
                            }
                            "INTEGER UNSIGNED" => {
                                query = query.bind(None::<u32>);
                            }
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
                    Value::UInt8(v) => {
                        query = query.bind(*v);
                    }
                    Value::UInt16(v) => {
                        query = query.bind(*v);
                    }
                    Value::UInt32(v) => {
                        query = query.bind(*v);
                    }
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
                        "TINYINT UNSIGNED" => {
                            query = query.bind(None::<u8>);
                        }
                        "SMALLINT UNSIGNED" => {
                            query = query.bind(None::<u16>);
                        }
                        "INTEGER UNSIGNED" => {
                            query = query.bind(None::<u32>);
                        }
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
                }
            }

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
                    | Value::Null => inserted_ids.push(result.last_insert_id()),
                }
            } else {
                inserted_ids.push(result.last_insert_id());
            }
        }

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
}
