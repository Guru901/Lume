use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    Filter,
    database::DatabaseError,
    row::Row,
    schema::{Column, Schema, Value},
};

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

    pub fn filter<V>(mut self, column: &'static Column<V>, value: V) -> Self
    where
        V: Into<Value>,
    {
        self.filters.push(Filter {
            column_name: column.name.to_string(),
            value: value.into(),
        });
        self
    }
}

// Make Query a Future
impl<T: Schema> Future for Query<T> {
    type Output = Result<Vec<Row>, DatabaseError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rows = Vec::new();
        Poll::Ready(Ok(rows))
    }
}
