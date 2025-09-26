#![warn(missing_docs)]

use crate::{
    filter::{Filter, FilterType},
    schema::{Column, Value},
};

pub fn eq<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_name: column.name().to_string(),
        value: value.into(),
        filter_type: FilterType::Eq,
    }
}

pub fn ne<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_name: column.name().to_string(),
        value: value.into(),
        filter_type: FilterType::Neq,
    }
}

pub fn gt<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_name: column.name().to_string(),
        value: value.into(),
        filter_type: FilterType::Gt,
    }
}

pub fn gte<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_name: column.name().to_string(),
        value: value.into(),
        filter_type: FilterType::Gte,
    }
}

pub fn lt<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_name: column.name().to_string(),
        value: value.into(),
        filter_type: FilterType::Lt,
    }
}

pub fn lte<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_name: column.name().to_string(),
        value: value.into(),
        filter_type: FilterType::Lte,
    }
}
