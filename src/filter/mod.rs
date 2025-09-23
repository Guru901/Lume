use crate::schema::Value;

mod filters;

pub use filters::*;

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

impl FilterType {
    pub(crate) fn to_sql(&self) -> &'static str {
        match self {
            FilterType::Eq => "=",
            FilterType::Neq => "!=",
            FilterType::In => "IN",
            FilterType::Gt => ">",
            FilterType::Lt => "<",
            FilterType::Gte => ">=",
            FilterType::Lte => "<=",
        }
    }
}

#[derive(Debug)]
pub struct Filter {
    pub(crate) column_name: String,
    pub(crate) value: Value,
    pub(crate) filter_type: FilterType,
}
