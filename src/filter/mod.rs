#![warn(missing_docs)]

//! # Filter Module
//!
//! This module provides query filtering functionality for building WHERE clauses.
//! It includes filter types and conditions for type-safe query building.

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
#[derive(Debug)]
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
///     column_name: "age".to_string(),
///     filter_type: FilterType::Gt,
///     value: Value::Int(18),
/// };
/// ```
#[derive(Debug)]
pub struct Filter {
    /// The name of the column to filter on
    pub column_name: String,
    /// The value to compare against
    pub value: Value,
    /// The type of comparison to perform
    pub filter_type: FilterType,
}

impl Filter {
    /// Creates a new filter with the given parameters.
    ///
    /// # Arguments
    ///
    /// - `column_name`: The name of the column to filter on
    /// - `filter_type`: The type of comparison to perform
    /// - `value`: The value to compare against
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::filter::{Filter, FilterType};
    /// use lume::schema::Value;
    ///
    /// let filter = Filter::new("age".to_string(), FilterType::Gt, Value::Int(18));
    /// ```
    pub fn new(column_name: String, filter_type: FilterType, value: Value) -> Self {
        Filter {
            column_name,
            filter_type,
            value,
        }
    }
}
