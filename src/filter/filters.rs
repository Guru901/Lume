#![warn(missing_docs)]

use crate::{
    filter::{Filter, FilterType},
    schema::{Column, Value},
};

/// Creates an equality filter (`=`) for the specified column and value.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `value` - The value to compare against. Can be any type that converts into [`Value`].
///
/// # Returns
///
/// A [`Filter`] representing the equality condition.
///
/// # Example
///
/// ```
/// use lume::filter::eq;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///     }
/// }
///
/// let filter = eq(User::name(), "Alice");
/// ```
pub fn eq_value<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: column.name().to_string(),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Eq,
    }
}

pub fn eq_column<T>(column_1: &'static Column<T>, column_2: &'static Column<T>) -> Filter {
    Filter {
        column_one: column_1.name().to_string(),
        value: None,
        column_two: Some(column_2.name().to_string()),
        filter_type: FilterType::Eq,
    }
}

/// Creates a not-equal filter (`!=`) for the specified column and value.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `value` - The value to compare against. Can be any type that converts into [`Value`].
///
/// # Returns
///
/// A [`Filter`] representing the not-equal condition.
///
/// # Example
///
/// ```
/// use lume::filter::ne;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///     }
/// }
///
/// let filter = ne(User::age(), 30);
/// ```
pub fn ne<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: column.name().to_string(),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Neq,
    }
}

/// Creates a greater-than filter (`>`) for the specified column and value.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `value` - The value to compare against. Can be any type that converts into [`Value`].
///
/// # Returns
///
/// A [`Filter`] representing the greater-than condition.
///
/// # Example
///
/// ```
/// use lume::filter::gt;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///     }
/// }
///
/// let filter = gt(User::age(), 18);
/// ```
pub fn gt<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: column.name().to_string(),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Gt,
    }
}

/// Creates a greater-than-or-equal filter (`>=`) for the specified column and value.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `value` - The value to compare against. Can be any type that converts into [`Value`].
///
/// # Returns
///
/// A [`Filter`] representing the greater-than-or-equal condition.
///
/// # Example
///
/// ```
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///     }
/// }
///
/// use lume::filter::gte;
/// let filter = gte(User::age(), 18);
/// ```
pub fn gte<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: column.name().to_string(),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Gte,
    }
}

/// Creates a less-than filter (`<`) for the specified column and value.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `value` - The value to compare against. Can be any type that converts into [`Value`].
///
/// # Returns
///
/// A [`Filter`] representing the less-than condition.
///
/// # Example
///
/// ```
/// use lume::filter::lt;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///     }
/// }
///
/// let filter = lt(User::age(), 65);
/// ```
pub fn lt<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: column.name().to_string(),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Lt,
    }
}

/// Creates a less-than-or-equal filter (`<=`) for the specified column and value.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `value` - The value to compare against. Can be any type that converts into [`Value`].
///
/// # Returns
///
/// A [`Filter`] representing the less-than-or-equal condition.
///
/// # Example
///
/// ```
/// use lume::filter::lte;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///     }
/// }
///
/// let filter = lte(User::age(), 29);
/// ```
pub fn lte<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: column.name().to_string(),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Lte,
    }
}
