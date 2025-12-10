#![warn(missing_docs)]

//! # Column Module
//!
//! This module provides the core column definition functionality for Lume.
//! It includes the `Column<T>` struct for type-safe column definitions and
//! the `Value` enum for database value storage and conversion.

use std::fmt::{Debug, Display};

use crate::schema::{ColumnConstraint, ColumnValidators, DefaultValueEnum, GeneratedColumn};

/// A type-safe column definition with constraints and metadata.
///
/// The `Column<T>` struct represents a database column with compile-time type safety.
/// It stores column metadata including name, constraints, and default values.
///
/// # Type Parameters
///
/// - `T`: The Rust type that this column stores
///
/// # Features
///
/// - **Type Safety**: Compile-time type checking for column operations
/// - **Constraints**: Support for primary key, not null, unique, and indexed constraints
/// - **Default Values**: Type-safe default value specification
/// - **SQL Generation**: Automatic SQL type mapping and constraint generation
///
/// # Example
///
/// ```rust
/// use lume::schema::Column;
///
/// // Create a column with constraints
/// let id_col = Column::<i32>::new("id", "users")
///     .primary_key()
///     .not_null();
///
/// let name_col = Column::<String>::new("name", "users")
///     .not_null()
///     .unique()
///     .default_value("Anonymous".to_string());
///
/// // Access column properties
/// assert_eq!(id_col.name(), "id");
/// ```
#[derive(Clone, Debug)]
pub struct Column<T> {
    pub(crate) name: &'static str,
    default_value: Option<DefaultValueEnum<T>>,
    table_name: &'static str,
    comment: Option<&'static str>,
    charset: Option<&'static str>,
    collate: Option<&'static str>,
    validators: Vec<ColumnValidators>,
    constraints: Vec<ColumnConstraint>,
}

impl<T: Debug> Display for Column<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Column {{
    name: {},
    default_value: {:?},
    comment: {:?},
    charset: {:?},
    collate: {:?},
    table_name: {}
}}",
            self.name,
            self.default_value,
            self.comment,
            self.charset,
            self.collate,
            self.table_name
        )
    }
}

impl<T> Column<T> {
    /// Creates a new column with the given name and table name.
    ///
    /// By default, columns are nullable and have no constraints.
    ///
    /// # Arguments
    ///
    /// - `name`: The name of the column in the database
    /// - `table_name`: The name of the table this column belongs to
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("username", "users");
    /// assert_eq!(col.name(), "username");
    /// ```
    pub const fn new(name: &'static str, table_name: &'static str) -> Self {
        Self {
            name,
            default_value: None,
            table_name,
            comment: None,
            charset: None,
            collate: None,
            validators: Vec::new(),
            constraints: Vec::new(),
        }
    }

    /// Sets a default value for this column.
    ///
    /// # Arguments
    ///
    /// - `value`: The default value to set
    pub fn default_value<K: Into<T>>(mut self, value: K) -> Self {
        self.default_value = Some(DefaultValueEnum::Value(value.into()));
        self
    }

    /// Sets this column's default value to the current date/time.
    ///
    ///
    /// - For types like [`time::OffsetDateTime`] and [`time::Date`], this sets their default value
    ///   to `CURRENT_TIMESTAMP` or the SQL-appropriate current value.
    /// - On types where `CURRENT_TIMESTAMP` is not meaningful, this flag is ignored.
    /// ```
    pub fn default_now(mut self) -> Self {
        self.default_value = Some(DefaultValueEnum::CurrentTimestamp);
        self
    }

    /// Makes this column NOT NULL.
    pub fn not_null(mut self) -> Self {
        self.constraints.push(ColumnConstraint::NonNullable);
        self
    }

    /// Sets this column's default value to a random value (where supported).
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    /// use lume::schema::DefaultValueEnum;
    ///
    /// let col = Column::<String>::new("id", "users").default_random();
    /// match col.get_default() {
    ///     Some(DefaultValueEnum::Random) => println!("Random default set"),
    ///     _ => panic!("Expected Random default"),
    /// }
    /// ```
    ///
    /// What counts as "random" depends on the column type and the backend.
    /// - For string columns, this means a randomly generated string (such as UUID).
    /// - For integer columns, it may be a random or auto-incrementing value.
    ///
    /// Not all databases support truly random default values for all types.
    /// This flag is intended as a hint for builder and migration tools.
    pub fn default_random(mut self) -> Self {
        self.default_value = Some(DefaultValueEnum::Random);
        self
    }

    /// Marks this column as requiring a valid email address.
    ///
    /// This is a semantic hint for validation and UI. It does not enforce
    /// format checks in the database (unless paired with a `check`).
    pub fn email(mut self) -> Self {
        self.validators.push(ColumnValidators::Email);
        self
    }

    /// Marks this column as containing a link (URL).
    ///
    /// This is a semantic hint for validation and UI, but does not enforce
    /// link format at the database level (unless paired with a `check`).
    pub fn link(mut self) -> Self {
        self.validators.push(ColumnValidators::Url);
        self
    }

    /// Sets the minimum allowed length for this column's string values.
    pub fn min_len(mut self, min: i32) -> Self {
        self.validators.push(ColumnValidators::MinLen(min as usize));
        self
    }

    /// Sets the maximum allowed length for this column's string values.
    pub fn max_len(mut self, max: i32) -> Self {
        self.validators.push(ColumnValidators::MaxLen(max as usize));
        self
    }

    /// Sets the minimum allowed value for numeric columns.
    pub fn min(mut self, min: usize) -> Self {
        self.validators.push(ColumnValidators::Min(min));
        self
    }

    /// Sets the maximum allowed value for numeric columns.
    pub fn max(mut self, max: usize) -> Self {
        self.validators.push(ColumnValidators::Max(max));
        self
    }

    /// Adds a UNIQUE constraint to this column.
    pub fn unique(mut self) -> Self {
        self.constraints.push(ColumnConstraint::Unique);
        self
    }

    /// Makes this column a primary key.
    ///
    /// Primary keys are automatically set to NOT NULL.
    pub fn primary_key(mut self) -> Self {
        self.constraints.push(ColumnConstraint::PrimaryKey);
        self
    }

    /// Adds an index to this column.
    pub fn indexed(mut self) -> Self {
        self.constraints.push(ColumnConstraint::Indexed);
        self
    }

    /// Enables AUTO_INCREMENT on this column (MySQL).
    pub fn auto_increment(mut self) -> Self {
        self.constraints.push(ColumnConstraint::AutoIncrement);
        self
    }

    /// Sets a column comment (MySQL `COMMENT`).
    pub fn comment(mut self, comment: &'static str) -> Self {
        self.comment = Some(comment);
        self
    }

    /// Sets the character set for this column (MySQL `CHARACTER SET`).
    pub fn charset(mut self, charset: &'static str) -> Self {
        self.charset = Some(charset);
        self
    }

    /// Sets the collation for this column (MySQL `COLLATE`).
    pub fn collate(mut self, collate: &'static str) -> Self {
        self.collate = Some(collate);
        self
    }

    /// Adds `ON UPDATE CURRENT_TIMESTAMP` behavior (MySQL).
    pub fn on_update_current_timestamp(mut self) -> Self {
        self.constraints
            .push(ColumnConstraint::OnUpdateCurrentTimestamp);
        self
    }

    /// Marks the column as INVISIBLE (MySQL 8).
    pub fn invisible(mut self) -> Self {
        self.constraints.push(ColumnConstraint::Invisible);
        self
    }

    /// Adds a CHECK constraint expression (MySQL 8).
    pub fn check(mut self, expression: &'static str) -> Self {
        self.constraints.push(ColumnConstraint::Check(expression));
        self
    }

    /// Defines this column as a VIRTUAL generated column (MySQL) with the given expression.
    pub fn generated_virtual(mut self, expression: &'static str) -> Self {
        self.constraints
            .push(ColumnConstraint::Generated(GeneratedColumn::Virtual(
                expression,
            )));
        self
    }

    /// Defines this column as a STORED generated column (MySQL) with the given expression.
    pub fn generated_stored(mut self, expression: &'static str) -> Self {
        self.constraints
            .push(ColumnConstraint::Generated(GeneratedColumn::Stored(
                expression,
            )));
        self
    }

    #[doc(hidden)]
    pub fn __internal_name(&self) -> &'static str {
        self.name
    }

    #[doc(hidden)]
    pub fn __internal_table_name(&self) -> &'static str {
        self.table_name
    }

    #[doc(hidden)]
    pub fn __internal_get_default(&self) -> Option<&DefaultValueEnum<T>> {
        self.default_value.as_ref()
    }

    #[doc(hidden)]
    pub fn __internal_get_validators(&self) -> Vec<ColumnValidators> {
        return self.validators.clone();
    }

    #[doc(hidden)]
    pub fn __internal_get_constraints(&self) -> Vec<ColumnConstraint> {
        return self.constraints.clone();
    }

    #[doc(hidden)]
    pub fn __internal_get_comment(&self) -> Option<&'static str> {
        self.comment
    }

    #[doc(hidden)]
    pub fn __internal_get_charset(&self) -> Option<&'static str> {
        self.charset
    }

    #[doc(hidden)]
    pub fn __internal_get_collate(&self) -> Option<&'static str> {
        self.collate
    }
}
