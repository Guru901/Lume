#![warn(missing_docs)]

use crate::database::DatabaseError;
use crate::schema::{Schema, Value};
use sqlx::MySqlPool;
use std::fmt::Debug;
use std::sync::Arc;

pub struct Insert<T: Schema + Debug> {
    data: T,
    conn: Arc<MySqlPool>,
    returning: bool,
}

impl<T: Schema + Debug> Insert<T> {
    pub fn new(data: T, conn: Arc<MySqlPool>) -> Self {
        Self {
            data,
            conn,
            returning: false,
        }
    }

    pub fn returning(mut self) -> Self {
        self.returning = true;
        self
    }

    pub async fn execute(self) -> Result<(), DatabaseError> {
        let mut sql = format!("INSERT INTO `{}` (", T::table_name());
        let mut conn = self.conn.acquire().await?;

        for (i, col) in T::get_all_columns().iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&col.name);
        }
        sql.push_str(") VALUES (");

        for (i, col) in T::get_all_columns().iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str("?");
        }
        sql.push_str(")");

        let mut query = sqlx::query(&sql);
        let values = self.data.values();
        let columns = T::get_all_columns();

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
                Value::Int(v) => {
                    query = query.bind(*v);
                }
                Value::Float(v) => {
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
                Value::Long(v) => {
                    query = query.bind(*v);
                }
            }
        }

        query.execute(&mut *conn).await?;

        Ok(())
    }
}
