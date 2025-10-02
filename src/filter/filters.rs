#![warn(missing_docs)]

use std::fmt::Debug;

use crate::{
    filter::{AndFilter, ArrayFilter, Filter, FilterType, Filtered, NotFilter, OrFilter},
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
/// use lume::filter::eq_value;
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
/// let filter = eq_value(User::name(), "Alice");
/// ```
pub fn eq_value<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: (column.table_name().to_string(), column.name().to_string()),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Eq,
    }
}

/// Creates an equality filter (`=`) for joining two columns.
///
/// This function is used for join conditions where you want to compare
/// two columns from different tables.
///
/// # Arguments
///
/// * `column_1` - The first column to compare (from the main table)
/// * `column_2` - The second column to compare (from the joined table)
///
/// # Returns
///
/// A [`Filter`] representing the column-to-column equality condition.
///
/// # Example
///
/// ```rust
/// use lume::filter::eq_column;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
///     Post {
///         id: i32 [primary_key()],
///         user_id: i32 [not_null()],
///         title: String [not_null()],
///     }
/// }
///
/// let join_filter = eq_column(User::id(), Post::user_id());
/// ```
pub fn eq_column<T>(column_1: &'static Column<T>, column_2: &'static Column<T>) -> Filter {
    Filter {
        column_one: (
            column_1.table_name().to_string(),
            column_1.name().to_string(),
        ),
        value: None,
        column_two: Some((
            column_2.table_name().to_string(),
            column_2.name().to_string(),
        )),
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
/// use lume::filter::ne_value;
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
/// let filter = ne_value(User::age(), 30);
/// ```
pub fn ne_value<T, V>(column: &'static Column<T>, value: V) -> Filter
where
    V: Into<Value>,
{
    Filter {
        column_one: (column.table_name().to_string(), column.name().to_string()),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Neq,
    }
}

/// Creates a not-equal filter (`!=`) comparing two columns.
///
/// This function generates a filter that checks whether the value in `column1`
/// is not equal to the value in `column2`. This is useful for queries where you
/// want to compare the values of two columns within the same row.
///
/// # Arguments
///
/// * `column1` - The first column to compare.
/// * `column2` - The second column to compare against.
///
/// # Returns
///
/// A [`Filter`] representing the not-equal condition between two columns.
///
/// # Example
///
/// ```
/// use lume::filter::ne_column;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///         age: i32,
///         other_age: i32,
///     }
/// }
///
/// let filter = ne_column(User::age(), User::other_age());
/// ```
pub fn ne_column<T>(column1: &'static Column<T>, column2: &'static Column<T>) -> Filter {
    Filter {
        column_one: (column1.table_name().to_string(), column1.name().to_string()),
        value: None,
        column_two: Some((column2.table_name().to_string(), column2.name().to_string())),
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
        column_one: (column.table_name().to_string(), column.name().to_string()),
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
        column_one: (column.table_name().to_string(), column.name().to_string()),
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
        column_one: (column.table_name().to_string(), column.name().to_string()),
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
        column_one: (column.table_name().to_string(), column.name().to_string()),
        value: Some(value.into()),
        column_two: None,
        filter_type: FilterType::Lte,
    }
}

/// Combines two filters with a logical OR, producing a filter that matches if either condition is true.
///
/// This function is useful for constructing complex query conditions where you want to match
/// records that satisfy at least one of the provided filters.
///
/// # Arguments
///
/// * `filter1` - The first filter condition.
/// * `filter2` - The second filter condition.
///
/// # Returns
///
/// An [`OrFilter`] representing the logical OR of the two filters.
///
/// # Example
///
/// ```
/// use lume::filter::{or, eq_value, lte};
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
/// let filter = or(
///     eq_value(User::name(), "Alice"),
///     lte(User::age(), 18)
/// );
/// ```
pub fn or(filter1: impl Filtered + 'static, filter2: impl Filtered + 'static) -> OrFilter {
    OrFilter {
        filter1: Box::new(filter1),
        filter2: Box::new(filter2),
    }
}

/// Combines two filters with a logical AND, producing a filter that matches if both conditions are true.
///
/// This function is useful for constructing complex query conditions where you want to match
/// records that satisfy both of the provided filters.
///
/// # Arguments
///
/// * `filter1` - The first filter condition.
/// * `filter2` - The second filter condition.
///
/// # Returns
///
/// An [`AndFilter`] representing the logical AND of the two filters.
///
/// # Example
///
/// ```
/// use lume::filter::{and, eq_value, lte};
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
/// let filter = and(
///     eq_value(User::name(), "Alice"),
///     lte(User::age(), 18)
/// );
/// ```
pub fn and(filter1: impl Filtered + 'static, filter2: impl Filtered + 'static) -> AndFilter {
    AndFilter {
        filter1: Box::new(filter1),
        filter2: Box::new(filter2),
    }
}

/// Negates a filter condition, producing a filter that matches when the given filter does not.
///
/// This function wraps an existing filter and inverts its logic, allowing you to express
/// queries such as "NOT (condition)" in SQL.
///
/// # Arguments
///
/// * `filter` - The filter condition to be negated. This can be any type that implements the [`Filtered`] trait.
///
/// # Returns
///
/// A [`NotFilter`] representing the logical negation of the provided filter.
///
/// # Example
///
/// ```
/// use lume::filter::{not, eq_value};
/// use lume::define_schema;
/// use lume::schema::Schema;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
/// }
///
/// let filter = not(eq_value(User::name(), "Alice"));
/// // This will generate a SQL condition like: NOT (users.name = 'Alice')
/// ```
pub fn not(filter: impl Filtered + 'static) -> NotFilter {
    NotFilter {
        filter: Box::new(filter),
    }
}

/// Creates a filter that matches rows where the column's value is contained in the given array of values.
///
/// This is equivalent to a SQL `IN` clause. The filter will match if the column's value is equal to
/// any of the values in the provided array.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `values` - The array of values to match against.
///
/// # Returns
///
/// An object implementing [`Filtered`] that represents the `IN` filter.
///
/// # Example
///
/// ```
/// use lume::filter::in_array;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String,
///     }
/// }
///
/// let IDS = &[Value::Int8(1), Value::Int8(2), Value::Int8(3)];
/// let filter = in_array(User::id(), IDS);
/// ```
pub fn in_array<T: Debug>(
    column: &'static Column<T>,
    values: &'static [Value],
) -> impl Filtered + 'static {
    ArrayFilter {
        column: Some((column.table_name().to_string(), column.name().to_string())),
        values: values,
        in_array: true,
    }
}

/// Creates a filter that matches rows where the column's value is *not* contained in the given array of values.
///
/// This is equivalent to a SQL `NOT IN` clause. The filter will match if the column's value is not equal to
/// any of the values in the provided array.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `values` - The array of values to exclude.
///
/// # Returns
///
/// An object implementing [`Filtered`] that represents the `NOT IN` filter.
///
/// # Example
///
/// ```
/// use lume::filter::not_in_array;
/// use lume::define_schema;
/// use lume::schema::ColumnInfo;
/// use lume::schema::Schema;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String,
///     }
/// }
///
/// let IDS = &[Value::Int8(1), Value::Int8(2), Value::Int8(3)];
/// let filter = not_in_array(User::id(), IDS);
/// ```
pub fn not_in_array<T: Debug>(
    column: &'static Column<T>,
    values: &'static [Value],
) -> impl Filtered + 'static {
    ArrayFilter {
        column: Some((column.table_name().to_string(), column.name().to_string())),
        values: values,
        in_array: false,
    }
}

/// Creates a filter that matches rows where the column's value is `NULL`.
///
/// This is equivalent to a SQL `IS NULL` clause. The filter will match if the column's value is `NULL`.
///
/// # Arguments
///
/// * `column` - The column to filter on.
///
/// # Returns
///
/// An object implementing [`Filtered`] that represents the `IS NULL` filter.
///
/// # Example
///
/// ```
/// use lume::filter::is_null;
/// use lume::define_schema;
/// use lume::schema::ColumnInfo;
/// use lume::schema::Schema;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String,
///     }
/// }
///
/// let filter = is_null(User::name());
/// ```
pub fn is_null<T: Debug>(column: &'static Column<T>) -> impl Filtered + 'static {
    Filter {
        column_one: (column.table_name().to_string(), column.name().to_string()),
        value: Some(Value::Null),
        column_two: None,
        filter_type: FilterType::Eq,
    }
}

/// Creates a filter that matches rows where the column's value is *not* `NULL`.
///
/// This is equivalent to a SQL `IS NOT NULL` clause. The filter will match if the column's value is not `NULL`.
///
/// # Arguments
///
/// * `column` - The column to filter on.
///
/// # Returns
///
/// An object implementing [`Filtered`] that represents the `IS NOT NULL` filter.
///
/// # Example
///
/// ```
/// use lume::filter::is_not_null;
/// use lume::define_schema;
/// use lume::schema::ColumnInfo;
/// use lume::schema::Schema;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String,
///     }
/// }
///
/// let filter = is_not_null(User::name());
/// ```
pub fn is_not_null<T: Debug>(column: &'static Column<T>) -> impl Filtered + 'static {
    Filter {
        column_one: (column.table_name().to_string(), column.name().to_string()),
        value: Some(Value::Null),
        column_two: None,
        filter_type: FilterType::Neq,
    }
}

/// Creates a filter that matches rows where the column's value is like the given pattern.
///
/// This is equivalent to a SQL `LIKE` clause. The filter will match if the column's value is like the given pattern.
///
/// # Arguments
///
/// * `column` - The column to filter on.
/// * `pattern` - The pattern to match.
///
/// # Returns
///
/// An object implementing [`Filtered`] that represents the `LIKE` filter.
///
/// # Example
///
/// ```
/// use lume::filter::like;
/// use lume::define_schema;
/// use lume::schema::ColumnInfo;
/// use lume::schema::Schema;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String,
///     }
/// }
///
/// let filter = like(User::name(), "%doe%");
/// ```
pub fn like<T: Debug>(
    column: &'static Column<T>,
    pattern: &'static str,
) -> impl Filtered + 'static {
    Filter {
        column_one: (column.table_name().to_string(), column.name().to_string()),
        value: Some(Value::String(pattern.to_string())),
        column_two: None,
        filter_type: FilterType::Like,
    }
}
