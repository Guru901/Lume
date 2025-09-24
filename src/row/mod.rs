use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use sqlx::mysql::MySqlRow;

use crate::schema::{Column, ColumnInfo, Schema, Value};

#[derive(Debug)]
pub struct Row<S: Schema + Debug> {
    data: std::collections::HashMap<String, Value>,
    _phanton: PhantomData<S>,
}

impl<S: Schema + Debug> Row<S> {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
            _phanton: PhantomData,
        }
    }

    pub fn insert<T>(&mut self, column: ColumnInfo, value: T)
    where
        T: Into<Value>,
    {
        self.data.insert(column.name.to_string(), value.into());
    }

    // Type-safe getter - returns the exact type expected
    pub fn get<T>(&self, column: &'static Column<T>) -> Option<T>
    where
        T: TryFrom<Value>,
    {
        self.data
            .get(column.name)
            .and_then(|v| T::try_from(v.clone()).ok())
    }

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

                println!("{:?}: {:?}", column.name, value);

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
