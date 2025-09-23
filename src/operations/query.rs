use std::{
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::filter::Filter;
use crate::{
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

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn select(self) -> Self {
        self
    }
}

// Make Query a Future
impl<T: Schema> Future for Query<T> {
    type Output = Result<Vec<Row<T>>, DatabaseError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rows = Vec::new();
        Poll::Ready(Ok(rows))
    }
}
