use crate::{operations::query::Query, schema::Schema};

pub struct Database;

impl Database {
    pub fn query<T: Schema>(&self) -> Query<T> {
        Query::new()
    }
}

#[derive(Debug)]
pub struct DatabaseError;

pub fn connect(_url: &str) -> Database {
    Database
}
