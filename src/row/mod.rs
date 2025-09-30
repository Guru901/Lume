#![warn(missing_docs)]

//! # Row Module
//!
//! This module provides the row abstraction for type-safe database data access.
//! The `Row<S>` struct represents a single row from a database table with
//! compile-time type safety for column access.

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use sqlx::mysql::{MySqlQueryResult, MySqlRow};

use crate::{
    operations::query::JoinInfo,
    schema::{Column, ColumnInfo, Schema, Value},
};

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
pub struct Row<S: Schema + Debug> {
    /// The row data stored as key-value pairs
    data: std::collections::HashMap<String, Value>,
    /// Phantom data to maintain schema type information
    _phanton: PhantomData<S>,
}

impl<S: Schema + Debug> Debug for Row<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Row").field("data", &self.data).finish()
    }
}

impl<S: Schema + Debug> Row<S> {
    /// Creates a new empty row.
    pub(crate) fn _new() -> Self {
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
    pub(crate) fn _insert<T>(&mut self, column: ColumnInfo, value: T)
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
    /// ```
    ///
    /// This would typically be called with MySQL query results
    /// let mysql_rows: Vec<MySqlRow> = sqlx::query("SELECT * FROM users").fetch_all(&pool).await?;
    /// let lume_rows: Vec<Row<User>> = Row::from_mysql_row(mysql_rows);
    pub(crate) fn from_mysql_row(rows: Vec<MySqlRow>, joins: Option<&Vec<JoinInfo>>) -> Vec<Self> {
        let mut rows_: Vec<Self> = Vec::new();

        for row in rows {
            let mut map = HashMap::new();

            // Extract columns from the main table
            let main_columns = S::get_all_columns();
            for column in main_columns {
                let value = Self::extract_column_value(&row, &column.name, &column.data_type);
                if let Some(value) = value {
                    map.insert(column.name.to_string(), value);
                }
            }

            if joins.is_some() {
                for join in joins.unwrap() {
                    let joined_column = &join.columns;

                    for column in joined_column {
                        let value =
                            Self::extract_column_value(&row, &column.name, &column.data_type);
                        if let Some(value) = value {
                            if map.contains_key(column.name) {
                                let fq_key = format!("{}.{}", join.table_name, column.name);
                                map.entry(fq_key).or_insert(value);
                            } else {
                                map.insert(column.name.to_string(), value);
                            }
                        }
                    }
                }
            }

            rows_.push(Self {
                data: map,
                _phanton: PhantomData,
            });
        }

        rows_
    }

    pub(crate) fn from_mysql_result(
        result: MySqlQueryResult,
        returning: &Vec<&'static str>,
    ) -> Vec<Self> {
        let mut rows_ = Vec::new();

        let last_id = result.last_insert_id();
        for col in returning.iter() {
            let map: HashMap<String, Value> = vec![(col.to_string(), Value::UInt64(last_id))]
                .into_iter()
                .collect();

            rows_.push(Self {
                data: map,
                _phanton: PhantomData,
            });
        }

        rows_
    }

    /// Extracts a column value from a MySQL row based on column name and data type
    fn extract_column_value(row: &MySqlRow, column_name: &str, data_type: &str) -> Option<Value> {
        use sqlx::Row as _;
        match data_type {
            "TEXT" => {
                // Try to get as string first
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
            "TINYINT" => {
                if let Ok(val) = row.try_get::<i8, _>(column_name) {
                    Some(Value::Int8(val))
                } else if let Ok(val) = row.try_get::<Option<i8>, _>(column_name) {
                    val.map(Value::Int8)
                } else {
                    None
                }
            }
            "SMALLINT" => {
                if let Ok(val) = row.try_get::<i16, _>(column_name) {
                    Some(Value::Int16(val))
                } else if let Ok(val) = row.try_get::<Option<i16>, _>(column_name) {
                    val.map(Value::Int16)
                } else {
                    None
                }
            }
            "INTEGER" => {
                if let Ok(val) = row.try_get::<i32, _>(column_name) {
                    Some(Value::Int32(val))
                } else if let Ok(val) = row.try_get::<Option<i32>, _>(column_name) {
                    val.map(Value::Int32)
                } else {
                    None
                }
            }
            "BIGINT" => {
                if let Ok(val) = row.try_get::<i64, _>(column_name) {
                    Some(Value::Int64(val))
                } else if let Ok(val) = row.try_get::<Option<i64>, _>(column_name) {
                    val.map(Value::Int64)
                } else {
                    None
                }
            }
            "TINYINT UNSIGNED" => {
                if let Ok(val) = row.try_get::<u8, _>(column_name) {
                    Some(Value::UInt8(val))
                } else if let Ok(val) = row.try_get::<Option<u8>, _>(column_name) {
                    val.map(Value::UInt8)
                } else {
                    None
                }
            }
            "SMALLINT UNSIGNED" => {
                if let Ok(val) = row.try_get::<u16, _>(column_name) {
                    Some(Value::UInt16(val))
                } else if let Ok(val) = row.try_get::<Option<u16>, _>(column_name) {
                    val.map(Value::UInt16)
                } else {
                    None
                }
            }
            "INTEGER UNSIGNED" => {
                if let Ok(val) = row.try_get::<u32, _>(column_name) {
                    Some(Value::UInt32(val))
                } else if let Ok(val) = row.try_get::<Option<u32>, _>(column_name) {
                    val.map(Value::UInt32)
                } else {
                    None
                }
            }
            "BIGINT UNSIGNED" => {
                if let Ok(val) = row.try_get::<u64, _>(column_name) {
                    Some(Value::UInt64(val))
                } else if let Ok(val) = row.try_get::<Option<u64>, _>(column_name) {
                    val.map(Value::UInt64)
                } else {
                    None
                }
            }
            "FLOAT" => {
                if let Ok(val) = row.try_get::<f32, _>(column_name) {
                    Some(Value::Float32(val))
                } else if let Ok(val) = row.try_get::<Option<f32>, _>(column_name) {
                    val.map(Value::Float32)
                } else {
                    None
                }
            }
            "REAL" | "DOUBLE PRECISION" | "DOUBLE" => {
                if let Ok(val) = row.try_get::<f64, _>(column_name) {
                    Some(Value::Float64(val))
                } else if let Ok(val) = row.try_get::<Option<f64>, _>(column_name) {
                    val.map(Value::Float64)
                } else {
                    None
                }
            }
            "BOOLEAN" => {
                if let Ok(val) = row.try_get::<bool, _>(column_name) {
                    Some(Value::Bool(val))
                } else if let Ok(val) = row.try_get::<Option<bool>, _>(column_name) {
                    val.map(Value::Bool)
                } else {
                    None
                }
            }
            _ => {
                // Fallback: try to get as string
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
        }
    }
}
