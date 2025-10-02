#![warn(missing_docs)]

//! # Filter Module
//!
//! This module provides query filtering functionality for building WHERE clauses.
//! It includes filter types and conditions for type-safe query building.

use std::fmt::Debug;

use crate::schema::Value;

mod filters;

pub use filters::*;

/// Enum representing different types of filter conditions for WHERE clauses.
///
/// This enum provides SQL operators for building query conditions.
///
/// # Variants
///
/// - `Eq`: Equality (=)
/// - `Neq`: Not equal (!=)
/// - `Gt`: Greater than (>)
/// - `Gte`: Greater than or equal (>=)
/// - `Lt`: Less than (<)
/// - `Lte`: Less than or equal (<=)
/// - `In`: IN clause (currently unused)
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FilterType {
    /// Equality operator (=)
    Eq,
    /// Not equal operator (!=)
    Neq,
    /// IN clause operator (currently unused)
    In,
    /// Greater than operator (>)
    Gt,
    /// Less than operator (<)
    Lt,
    /// Greater than or equal operator (>=)
    Gte,
    /// Less than or equal operator (<=)
    Lte,
    /// OR operator (logical OR)
    Or,
    /// AND operator (logical AND)
    And,
    /// LIKE operator (LIKE)
    Like,
    /// NOT operator (NOT)
    Not,
}

impl FilterType {
    /// Converts the filter type to its SQL operator string.
    ///
    /// # Returns
    ///
    /// The SQL operator string for this filter type
    pub(crate) fn to_sql(&self) -> &'static str {
        match self {
            FilterType::Eq => "=",
            FilterType::Neq => "!=",
            FilterType::In => "IN",
            FilterType::Gt => ">",
            FilterType::Lt => "<",
            FilterType::Gte => ">=",
            FilterType::Lte => "<=",
            FilterType::Or => "OR",
            FilterType::And => "AND",
            FilterType::Like => "LIKE",
            FilterType::Not => "NOT",
        }
    }
}

/// Represents a filter condition for query WHERE clauses.
///
/// This struct combines a column name, filter type, and value to create
/// a condition that can be used in database queries.
///
/// # Fields
///
/// - `column_name`: The name of the column to filter on
/// - `filter_type`: The type of comparison to perform
/// - `value`: The value to compare against
///
/// # Example
///
/// ```rust
/// use lume::filter::Filter;
/// use lume::schema::Value;
/// use lume::filter::FilterType;
///
/// let filter = Filter {
///     column_one: ("users".to_string(), "age".to_string()),
///     filter_type: FilterType::Gt,
///     value: Some(Value::Int8(18)),
///     column_two: None,
/// };
/// ```
#[derive(Debug)]
pub struct Filter {
    /// The name of the column to filter on
    pub column_one: (String, String),
    /// The value to compare against
    pub value: Option<Value>,
    /// The name of the column to filter on (for joins)
    pub column_two: Option<(String, String)>,
    /// The type of comparison to perform
    pub filter_type: FilterType,
}

/// Represents 'OR'  filter condition for query WHERE clauses.
///
/// This struct combines two filter conditions to create
/// a condition that can be used in database queries.
///
/// # Fields
///
/// - `filter1`: The first filter condition
/// - `filter2`: The second filter condition
///
/// # Example
///
/// ```rust
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
///     lte(User::age(), 30)
/// );
/// ```
#[derive(Debug)]
pub struct OrFilter {
    pub(crate) filter1: Box<dyn Filtered>,
    pub(crate) filter2: Box<dyn Filtered>,
}

/// Represents 'AND'  filter condition for query WHERE clauses.
///
/// This struct combines two filter conditions to create
/// a condition that can be used in database queries.
///
/// # Fields
///
/// - `filter1`: The first filter condition
/// - `filter2`: The second filter condition
///
/// # Example
///
/// ```rust
/// use lume::filter::{and, eq_value, lt};
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
///     lt(User::age(), 30)
/// );
/// ```
#[derive(Debug)]
pub struct AndFilter {
    pub(crate) filter1: Box<dyn Filtered>,
    pub(crate) filter2: Box<dyn Filtered>,
}

/// Represents a logical 'NOT' filter condition for query WHERE clauses.
///
/// This struct wraps another filter condition and negates it, allowing you to
/// express queries such as "NOT (condition)" in SQL.
///
/// # Fields
///
/// - `filter`: The filter condition to be negated. This is any type that implements the [`Filtered`] trait.
///
/// # Example
///
/// ```rust
/// use lume::filter::{not, eq_value};
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
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
#[derive(Debug)]
pub struct NotFilter {
    /// The filter condition to be negated.
    pub(crate) filter: Box<dyn Filtered>,
}

/// Represents a filter for checking if a column's value is (or is not) in a given array of values.
///
/// This struct is used to build SQL `IN` or `NOT IN` conditions for queries, allowing you to
/// filter rows where a column matches any value in a provided array.
///
/// # Fields
///
/// - `column`: The column to filter on, represented as an optional tuple of (table, column) names.
/// - `values`: A static slice of `Value` items to compare against the column.
/// - `in_array`: If `true`, generates an `IN` filter; if `false`, generates a `NOT IN` filter.
///
#[derive(Debug)]
pub struct ArrayFilter {
    /// The column to filter on, as (table, column) or None.
    pub(crate) column: Option<(String, String)>,
    /// The array of values to compare against.
    pub(crate) values: &'static [Value],
    /// Whether this is an `IN` (true) or `NOT IN` (false) filter.
    pub(crate) in_array: bool,
}

/// Trait for all filter types used in query building.
///
/// This trait abstracts over different filter types (such as simple column-value filters,
/// column-column filters, and logical combinators like AND/OR) to allow uniform handling
/// of filters in query construction and evaluation.
///
/// Implementors of this trait provide access to filter details such as the value being compared,
/// the columns involved, the filter type (e.g., equality, less-than), and whether the filter
/// is a logical combinator (AND/OR).
pub trait Filtered: Debug {
    /// Returns a reference to the value being compared in the filter, if any.
    ///
    /// For simple column-value filters, this returns `Some(&Value)`.
    /// For logical combinators (AND/OR), this returns `None`.
    fn value(&self) -> Option<&Value>;

    /// Returns a reference to the first column involved in the filter, if any.
    ///
    /// For simple filters, this is the column being filtered.
    /// For logical combinators (AND/OR), this returns `None`.
    fn column_one(&self) -> Option<&(String, String)>;

    /// Returns a reference to the second column involved in the filter, if any.
    ///
    /// This is used for column-to-column comparisons (e.g., joins).
    /// For most filters, this is `None`.
    fn column_two(&self) -> Option<&(String, String)>;

    /// Returns the type of filter (e.g., Eq, Lt, Gt, etc.).
    fn filter_type(&self) -> FilterType;

    /// Returns `true` if this filter is a logical OR combinator.
    fn is_or_filter(&self) -> bool;

    /// Returns `true` if this filter is a logical AND combinator.
    fn is_and_filter(&self) -> bool;

    /// Returns a reference to the first sub-filter if this is a logical combinator.
    ///
    /// For AND/OR filters, this returns `Some(&dyn Filtered)`.
    /// For simple filters, this returns `None`.
    fn filter1(&self) -> Option<&dyn Filtered>;

    /// Returns a reference to the second sub-filter if this is a logical combinator.
    ///
    /// For AND/OR filters, this returns `Some(&dyn Filtered)`.
    /// For simple filters, this returns `None`.
    fn filter2(&self) -> Option<&dyn Filtered>;

    /// Returns a reference to the array of values used in an array filter (e.g., IN/NOT IN), if any.
    ///
    /// For filters that operate on an array of values (such as SQL `IN` or `NOT IN` clauses),
    /// this returns `Some(&[Value])` containing the values being compared.
    /// For other filter types, this returns `None`.
    fn array_values(&self) -> Option<&[Value]>;

    /// Returns `Some(true)` for IN array filters, `Some(false)` for NOT IN, or `None` otherwise.
    fn is_in_array(&self) -> Option<bool> {
        None
    }

    /// Returns `Some(true)` if this filter is a logical NOT combinator, `None` otherwise.
    ///
    /// For filters that represent a logical NOT (negation), this should return `Some(true)`.
    /// For all other filters, this returns `None` by default.
    fn is_not(&self) -> Option<bool> {
        None
    }
}

impl Filtered for Filter {
    fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    fn column_one(&self) -> Option<&(String, String)> {
        Some(&self.column_one)
    }

    fn column_two(&self) -> Option<&(String, String)> {
        self.column_two.as_ref()
    }

    fn filter_type(&self) -> FilterType {
        self.filter_type
    }

    fn filter1(&self) -> Option<&dyn Filtered> {
        None
    }

    fn filter2(&self) -> Option<&dyn Filtered> {
        None
    }

    fn is_or_filter(&self) -> bool {
        false
    }

    fn is_and_filter(&self) -> bool {
        false
    }

    fn array_values(&self) -> Option<&[Value]> {
        None
    }

    fn is_in_array(&self) -> Option<bool> {
        None
    }
}

impl Filtered for OrFilter {
    fn value(&self) -> Option<&Value> {
        // if self.filter1.value.is_some() {
        //     self.filter1.value.as_ref()
        // } else {
        //     self.filter2.value.as_ref()
        // }
        None
    }

    fn column_one(&self) -> Option<&(String, String)> {
        None
    }

    fn column_two(&self) -> Option<&(String, String)> {
        None
    }

    fn filter_type(&self) -> FilterType {
        FilterType::Or
    }

    fn filter1(&self) -> Option<&dyn Filtered> {
        Some(&*self.filter1)
    }

    fn filter2(&self) -> Option<&dyn Filtered> {
        Some(&*self.filter2)
    }

    fn is_or_filter(&self) -> bool {
        true
    }

    fn is_and_filter(&self) -> bool {
        false
    }

    fn array_values(&self) -> Option<&[Value]> {
        None
    }

    fn is_in_array(&self) -> Option<bool> {
        None
    }
}

impl Filtered for AndFilter {
    fn value(&self) -> Option<&Value> {
        // if self.filter1.value.is_some() {
        //     self.filter1.value.as_ref()
        // } else {
        //     self.filter2.value.as_ref()
        // }
        None
    }

    fn column_one(&self) -> Option<&(String, String)> {
        None
    }

    fn column_two(&self) -> Option<&(String, String)> {
        None
    }

    fn filter1(&self) -> Option<&dyn Filtered> {
        Some(&*self.filter1)
    }

    fn filter2(&self) -> Option<&dyn Filtered> {
        Some(&*self.filter2)
    }

    fn is_or_filter(&self) -> bool {
        false
    }

    fn is_and_filter(&self) -> bool {
        true
    }

    fn filter_type(&self) -> FilterType {
        FilterType::And
    }

    fn array_values(&self) -> Option<&[Value]> {
        None
    }

    fn is_in_array(&self) -> Option<bool> {
        None
    }
}

impl Filtered for ArrayFilter {
    fn is_and_filter(&self) -> bool {
        false
    }

    fn value(&self) -> Option<&Value> {
        None
    }

    fn column_one(&self) -> Option<&(String, String)> {
        self.column.as_ref()
    }

    fn column_two(&self) -> Option<&(String, String)> {
        None
    }

    fn filter_type(&self) -> FilterType {
        FilterType::In
    }

    fn is_or_filter(&self) -> bool {
        false
    }

    fn filter1(&self) -> Option<&dyn Filtered> {
        None
    }

    fn filter2(&self) -> Option<&dyn Filtered> {
        None
    }

    fn array_values(&self) -> Option<&[Value]> {
        Some(self.values)
    }

    fn is_in_array(&self) -> Option<bool> {
        Some(self.in_array)
    }
}

impl Filtered for NotFilter {
    fn value(&self) -> Option<&Value> {
        None
    }

    fn column_one(&self) -> Option<&(String, String)> {
        None
    }

    fn column_two(&self) -> Option<&(String, String)> {
        None
    }

    fn filter_type(&self) -> FilterType {
        FilterType::Not
    }

    fn is_or_filter(&self) -> bool {
        false
    }

    fn is_and_filter(&self) -> bool {
        false
    }

    fn filter1(&self) -> Option<&dyn Filtered> {
        Some(&*self.filter)
    }

    fn filter2(&self) -> Option<&dyn Filtered> {
        None
    }

    fn array_values(&self) -> Option<&[Value]> {
        None
    }

    fn is_in_array(&self) -> Option<bool> {
        None
    }

    fn is_not(&self) -> Option<bool> {
        Some(true)
    }
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            value: Some(Value::Null),
            filter_type: FilterType::Eq,
            column_one: ("".to_string(), "".to_string()),
            column_two: None,
        }
    }
}
