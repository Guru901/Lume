use std::marker::PhantomData;

use crate::schema::{Column, Value};

#[derive(Debug)]
pub struct Row<S> {
    data: std::collections::HashMap<String, Value>,
    _phanton: PhantomData<S>,
}

impl<S> Row<S> {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
            _phanton: PhantomData,
        }
    }

    pub fn insert<T>(&mut self, column: Column<T>, value: T)
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
}
