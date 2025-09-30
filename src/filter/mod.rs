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
#[derive(Debug, PartialEq)]
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

    Or,
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

#[derive(Debug)]
pub struct OrFilter {
    pub(crate) filter1: Filter,
    pub(crate) filter2: Filter,
}

/// Trait for filter types that can be used in query WHERE clauses.
///
/// This trait provides a unified interface for both simple filters ([`Filter`])
/// and composite OR filters ([`OrFilter`]). It enables dynamic dispatch of
/// filter operations during query building.
///
/// # Contract
///
/// Implementors must ensure:
/// - When `is_or_filter()` returns `true`, both `filter1()` and `filter2()` must return `Some`
/// - When `is_or_filter()` returns `false`, both `filter1()` and `filter2()` must return `None`
/// - At least one of `value()` or `column_two()` should return `Some` for simple comparisons
pub trait Filtered: Debug {
    fn value(&self) -> Option<&Value>;
    fn column_one(&self) -> Option<&(String, String)>;
    fn column_two(&self) -> Option<&(String, String)>;
    fn filter_type(&self) -> FilterType;
    fn is_or_filter(&self) -> bool;

    fn filter1(&self) -> Option<&Filter>;
    fn filter2(&self) -> Option<&Filter>;
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

    fn filter1(&self) -> Option<&Filter> {
        None
    }

    fn filter2(&self) -> Option<&Filter> {
        None
    }

    fn is_or_filter(&self) -> bool {
        false
    }
}

impl Filtered for OrFilter {
    fn value(&self) -> Option<&Value> {
        if self.filter1.value.is_some() {
            self.filter1.value.as_ref()
        } else {
            self.filter2.value.as_ref()
        }
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

    fn filter1(&self) -> Option<&Filter> {
        Some(&self.filter1)
    }

    fn filter2(&self) -> Option<&Filter> {
        Some(&self.filter2)
    }

    fn is_or_filter(&self) -> bool {
        true
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
