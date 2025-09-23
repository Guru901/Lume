use std::{
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{database::DatabaseError, row::Row, schema::Schema};
use crate::{filter::Filter, schema::Value};

#[derive(Debug)]
pub struct Query<T> {
    table: PhantomData<T>,
    filters: Vec<Filter>,
}

impl<T: Schema> Query<T> {
    pub fn new() -> Self {
        Self {
            table: PhantomData,
            filters: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn select(self) -> Self {
        self
    }
}

// Make Query a Future
impl<T: Schema + Debug> Future for Query<T> {
    type Output = Result<Vec<Row<T>>, DatabaseError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut sql = format!("SELECT * FROM {}", T::table_name());

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

        println!("{}", sql);
        let rows = Vec::new();
        Poll::Ready(Ok(rows))
    }
}
