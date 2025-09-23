#![allow(dead_code)]

pub mod database;
pub mod operations;
pub mod row;
pub mod schema;

use crate::schema::Value;

#[derive(Debug)]
pub struct Filter {
    column_name: String,
    value: Value,
}
