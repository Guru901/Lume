use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use sqlx::MySqlPool;

use crate::{database::DatabaseError, row::Row, schema::Schema};
use crate::{filter::Filter, schema::Value};

#[derive(Debug)]
pub struct Query<T> {
    table: PhantomData<T>,
    filters: Vec<Filter>,
    conn: Arc<MySqlPool>,
}

impl<T: Schema + Debug> Query<T> {
    pub fn new(conn: Arc<MySqlPool>) -> Self {
        Self {
            table: PhantomData,
            filters: Vec::new(),
            conn,
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn select(self) -> Self {
        self
    }

    pub async fn execute(self) -> Result<Vec<Row<T>>, DatabaseError> {
        let mut sql = format!("SELECT * FROM {}", T::table_name());
        let mut conn = self.conn.acquire().await.unwrap();

        if !self.filters.is_empty() {
            let filter_sql = format!(" WHERE ");
            sql.push_str(&filter_sql);

            for (i, filter) in self.filters.iter().enumerate() {
                match &filter.value {
                    Value::String(_) => {
                        let filter_sql = format!(
                            "{} {} '{}' {}",
                            filter.column_name,
                            filter.filter_type.to_sql(),
                            filter.value,
                            if i == self.filters.len() - 1 {
                                ""
                            } else {
                                " AND "
                            }
                        );
                        sql.push_str(&filter_sql);
                    }
                    _ => {
                        let filter_sql = format!(
                            "{} {} {} {}",
                            filter.column_name,
                            filter.filter_type.to_sql(),
                            filter.value,
                            if i == self.filters.len() - 1 {
                                ""
                            } else {
                                " AND "
                            }
                        );

                        sql.push_str(&filter_sql);
                    }
                }
            }
        }

        let data = sqlx::query(&sql).fetch_all(&mut *conn).await.unwrap();
        let rows = Row::from_mysql_row(data);

        println!("{:#?}", rows[0]);

        Ok(rows)
    }
}
