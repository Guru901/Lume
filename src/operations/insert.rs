#![warn(missing_docs)]

//! # Insert Operation
//!
//! This module provides the [`Insert`] struct for type-safe insertion of records
//! into a MySQL database using a schema definition. It supports optional
//! returning of inserted rows and handles value binding for various SQL types.

use crate::database::DatabaseError;
use crate::schema::{Schema, Value};
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
pub struct Insert<T: Schema + Debug> {
    /// The data to be inserted.
    data: T,
    /// The database connection pool.
    conn: Arc<MySqlPool>,
    /// Whether to return the inserted row(s).
    returning: bool,
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
            returning: false,
        }
    }

    /// Configures the insert to return the inserted row(s).
    ///
    /// # Returns
    ///
    /// The [`Insert`] instance with returning enabled.
    pub fn returning(mut self) -> Self {
        self.returning = true;
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
    pub async fn execute(self) -> Result<(), DatabaseError> {
        let mut sql = format!("INSERT INTO `{}` (", T::table_name());
        let mut conn = self.conn.acquire().await?;

        // Build the column list for the INSERT statement.
        for (i, col) in T::get_all_columns().iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&col.name);
        }
        sql.push_str(") VALUES (");

        // Build the placeholders for the VALUES clause.
        for (i, _col) in T::get_all_columns().iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str("?");
        }
        sql.push_str(")");

        let mut query = sqlx::query(&sql);
        let values = self.data.values();
        let columns = T::get_all_columns();

        // Bind each value to the query, handling NULLs and type mapping.
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

        query.execute(&mut *conn).await?;

        Ok(())
    }
}
