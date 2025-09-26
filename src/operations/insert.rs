use crate::database::DatabaseError;
use crate::schema::{Schema, Value};
use sqlx::MySqlPool;
use std::fmt::Debug;
use std::sync::Arc;

pub struct Insert<T: Schema + Debug> {
    data: T,
    conn: Arc<MySqlPool>,
}

impl<T: Schema + Debug> Insert<T> {
    pub fn new(data: T, conn: Arc<MySqlPool>) -> Self {
        Self { data, conn }
    }

    pub async fn execute(self) -> Result<(), DatabaseError> {
        let mut sql = format!("INSERT INTO {} (", T::table_name());
        let mut conn = self.conn.acquire().await.unwrap();

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

        println!("{}", sql);

        let mut query = sqlx::query(&sql);
        let values = self.data.values();
        for col in T::get_all_columns().iter() {
            let value = values.get(col.name).unwrap();
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
                    query = query.bind(v);
                }
                Value::Null => {
                    query = query.bind(None::<i32>);
                }
                Value::Long(v) => {
                    query = query.bind(*v);
                }
            }
        }

        query.execute(&mut *conn).await?;
        Ok(())
    }
}
