//! # Row Module
//!
//! This module provides the row abstraction for type-safe database data access.
//! The `Row<S>` struct represents a single row from a database table with
//! compile-time type safety for column access.

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use sqlx::mysql::MySqlRow;

use crate::schema::{Column, ColumnInfo, Schema, Value};

/// A type-safe representation of a database row.
///
/// The `Row<S>` struct stores the data from a database row and provides
/// type-safe access to column values using the schema type `S`.
///
/// # Type Parameters
///
/// - `S`: The schema type that defines the table structure
///
/// # Features
///
/// - **Type Safety**: Compile-time type checking for column access
/// - **Flexible Storage**: Stores values in a type-erased format
/// - **MySQL Integration**: Built-in support for MySQL row extraction
/// - **Value Conversion**: Automatic conversion between database and Rust types
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
/// use lume::row::Row;
/// use lume::schema::{ColumnInfo, Schema};
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         email: String,
///     }
/// }
///
/// let mut row = Row::<User>::new();
///
/// // Insert data
/// row.insert(ColumnInfo {
///     name: "id",
///     data_type: "INTEGER",
///     nullable: false,
///     unique: false,
///     primary_key: true,
///     indexed: false,
///     has_default: false,
///     default_sql: None,
/// }, 42);
///
/// // Access data type-safely
/// let id_col = User::id();
/// let id: Option<i32> = row.get(id_col);
/// assert_eq!(id, Some(42));
/// ```
#[derive(Debug)]
pub struct Row<S: Schema + Debug> {
    /// The row data stored as key-value pairs
    data: std::collections::HashMap<String, Value>,
    /// Phantom data to maintain schema type information
    _phanton: PhantomData<S>,
}

impl<S: Schema + Debug> Row<S> {
    /// Creates a new empty row.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::define_schema;
    /// use lume::row::Row;
    /// use lume::schema::{Schema, ColumnInfo};
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///     }
    /// }
    ///
    /// let row = Row::<User>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
            _phanton: PhantomData,
        }
    }

    /// Inserts a value into this row for the specified column.
    ///
    /// # Arguments
    ///
    /// - `column`: Metadata about the column
    /// - `value`: The value to insert (must implement `Into<Value>`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::define_schema;
    /// use lume::row::Row;
    /// use lume::schema::{Schema, ColumnInfo};
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// let mut row = Row::<User>::new();
    /// row.insert(ColumnInfo {
    ///     name: "id",
    ///     data_type: "INTEGER",
    ///     nullable: false,
    ///     unique: false,
    ///     primary_key: true,
    ///     indexed: false,
    ///     has_default: false,
    ///     default_sql: None,
    /// }, 42);
    /// ```
    pub fn insert<T>(&mut self, column: ColumnInfo, value: T)
    where
        T: Into<Value>,
    {
        self.data.insert(column.name.to_string(), value.into());
    }

    /// Retrieves a value from this row for the specified column.
    ///
    /// This method provides type-safe access to column values. It returns
    /// `None` if the column doesn't exist or if the value can't be converted
    /// to the expected type.
    ///
    /// # Arguments
    ///
    /// - `column`: A reference to the column definition
    ///
    /// # Returns
    ///
    /// - `Some(T)`: The value if found and convertible
    /// - `None`: If the column doesn't exist or conversion fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::define_schema;
    /// use lume::row::Row;
    /// use lume::schema::{Schema, ColumnInfo};
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// let mut row = Row::<User>::new();
    /// row.insert(ColumnInfo {
    ///     name: "id",
    ///     data_type: "INTEGER",
    ///     nullable: false,
    ///     unique: false,
    ///     primary_key: true,
    ///     indexed: false,
    ///     has_default: false,
    ///     default_sql: None,
    /// }, 42);
    ///
    /// // Type-safe access
    /// let id_col = User::id();
    /// let id: Option<i32> = row.get(id_col);
    /// assert_eq!(id, Some(42));
    /// ```
    pub fn get<T>(&self, column: &'static Column<T>) -> Option<T>
    where
        T: TryFrom<Value>,
    {
        self.data
            .get(column.name)
            .and_then(|v| T::try_from(v.clone()).ok())
    }

    /// Converts MySQL rows to type-safe Lume rows.
    ///
    /// This method extracts data from MySQL rows and converts them to
    /// type-safe `Row<S>` instances. It uses the schema's column metadata
    /// to determine the correct types for extraction.
    ///
    /// # Arguments
    ///
    /// - `rows`: A vector of MySQL rows from a database query
    ///
    /// # Returns
    ///
    /// A vector of type-safe `Row<S>` instances
    ///
    /// # Type Extraction
    ///
    /// The method uses the schema's column metadata to determine how to
    /// extract values from MySQL rows:
    ///
    /// - `VARCHAR(255)` → `String`
    /// - `INTEGER` → `i32`
    /// - `BIGINT` → `i64`
    /// - `FLOAT`/`DOUBLE` → `f64`
    /// - `BOOLEAN` → `bool`
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::define_schema;
    /// use lume::row::Row;
    /// use lume::schema::{Schema, ColumnInfo};
    /// use sqlx::mysql::MySqlRow;
    ///
    /// define_schema! {
    ///     User {
    ///         id: i32 [primary_key()],
    ///         name: String [not_null()],
    ///     }
    /// }
    ///
    /// // This would typically be called with MySQL query results
    /// // let mysql_rows: Vec<MySqlRow> = sqlx::query("SELECT * FROM users").fetch_all(&pool).await?;
    /// // let lume_rows: Vec<Row<User>> = Row::from_mysql_row(mysql_rows);
    /// ```
    pub(crate) fn from_mysql_row(rows: Vec<MySqlRow>) -> Vec<Self> {
        use sqlx::Row as _;

        let mut rows_: Vec<Self> = Vec::new();

        for row in rows {
            let columns = S::get_all_columns();
            let mut map = HashMap::new();

            for column in columns {
                let value = match column.data_type {
                    "TEXT" => {
                        // Try to get as string first
                        if let Ok(val) = row.try_get::<String, _>(column.name) {
                            Some(Value::String(val))
                        } else if let Ok(val) = row.try_get::<Option<String>, _>(column.name) {
                            val.map(Value::String)
                        } else {
                            None
                        }
                    }
                    "INTEGER" => {
                        if let Ok(val) = row.try_get::<i32, _>(column.name) {
                            Some(Value::Int(val))
                        } else if let Ok(val) = row.try_get::<Option<i32>, _>(column.name) {
                            val.map(Value::Int)
                        } else {
                            None
                        }
                    }
                    "BIGINT" => {
                        if let Ok(val) = row.try_get::<i64, _>(column.name) {
                            Some(Value::Long(val))
                        } else if let Ok(val) = row.try_get::<Option<i64>, _>(column.name) {
                            val.map(Value::Long)
                        } else {
                            None
                        }
                    }
                    "REAL" | "DOUBLE PRECISION" => {
                        if let Ok(val) = row.try_get::<f64, _>(column.name) {
                            Some(Value::Float(val))
                        } else if let Ok(val) = row.try_get::<Option<f64>, _>(column.name) {
                            val.map(Value::Float)
                        } else {
                            None
                        }
                    }
                    "BOOLEAN" => {
                        if let Ok(val) = row.try_get::<bool, _>(column.name) {
                            Some(Value::Bool(val))
                        } else if let Ok(val) = row.try_get::<Option<bool>, _>(column.name) {
                            val.map(Value::Bool)
                        } else {
                            None
                        }
                    }
                    _ => {
                        // Fallback: try to get as string
                        if let Ok(val) = row.try_get::<String, _>(column.name) {
                            Some(Value::String(val))
                        } else if let Ok(val) = row.try_get::<Option<String>, _>(column.name) {
                            val.map(Value::String)
                        } else {
                            None
                        }
                    }
                };

                if let Some(value) = value {
                    map.insert(column.name.to_string(), value);
                }
            }

            rows_.push(Self {
                data: map,
                _phanton: PhantomData,
            });
        }

        rows_
    }
}
