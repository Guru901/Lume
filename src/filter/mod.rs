use crate::schema::{Column, Value};

mod filters;

pub use filters::eq;

#[derive(Debug)]
pub(crate) enum FilterType {
    Eq,  // Equals
    Neq, // Not Equals
    In,  // In
    Gt,  // Greater Than
    Lt,  // Less Than
    Gte, // Greater Than or Equals
    Lte, // Less Than or Equals
}

#[derive(Debug)]
pub struct Filter {
    pub(crate) column_name: String,
    pub(crate) value: Value,
    pub(crate) filter_type: FilterType,
}
