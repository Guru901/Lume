#![warn(missing_docs)]

//! # Column Module
//!
//! This module provides the core column definition functionality for Lume.
//! It includes the `Column<T>` struct for type-safe column definitions and
//! the `Value` enum for database value storage and conversion.

use std::{
    any::Any,
    fmt::{Debug, Display},
    marker::PhantomData,
};

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
/// assert!(id_col.is_primary_key());
/// assert!(!id_col.is_nullable());
/// assert_eq!(name_col.get_default(), Some(&"Anonymous".to_string()));
/// ```
#[derive(Clone, Debug)]
pub struct Column<T> {
    /// The column name in the database
    pub(crate) name: &'static str,
    /// The default value for this column
    default_value: Option<T>,
    /// The name of the table this column belongs to
    table_name: &'static str,
    /// Whether this column allows NULL values
    nullable: bool,
    /// Whether this column has a UNIQUE constraint
    unique: bool,
    /// Whether this column is a primary key
    primary_key: bool,
    /// Whether this column has an index
    indexed: bool,
    /// Phantom data to maintain type information
    _phantom: PhantomData<T>,
}

impl<T: Debug> Display for Column<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Column {{
    name: {},
    default_value: {:?},
    nullable: {},
    unique: {},
    primary_key: {},
    indexed: {},
    table_name: {}
}}",
            self.name,
            self.default_value,
            self.nullable,
            self.unique,
            self.primary_key,
            self.indexed,
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
    /// assert!(col.is_nullable());
    /// ```
    pub const fn new(name: &'static str, table_name: &'static str) -> Self {
        Self {
            name,
            default_value: None,
            nullable: true,
            table_name,
            unique: false,
            primary_key: false,
            indexed: false,
            _phantom: PhantomData,
        }
    }

    /// Sets a default value for this column.
    ///
    /// # Arguments
    ///
    /// - `value`: The default value to set
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("name", "users")
    ///     .default_value("Anonymous".to_string());
    ///
    /// assert_eq!(col.get_default(), Some(&"Anonymous".to_string()));
    /// ```
    pub fn default_value(mut self, value: T) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Makes this column NOT NULL.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("name", "users")
    ///     .not_null();
    ///
    /// assert!(!col.is_nullable());
    /// ```
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Adds a UNIQUE constraint to this column.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("email", "users")
    ///     .unique();
    ///
    /// assert!(col.is_unique());
    /// ```
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Makes this column a primary key.
    ///
    /// Primary keys are automatically set to NOT NULL.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<i32>::new("id", "users")
    ///     .primary_key();
    ///
    /// assert!(col.is_primary_key());
    /// assert!(!col.is_nullable()); // Primary keys are always NOT NULL
    /// ```
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false; // Primary keys are always not null
        self
    }

    /// Adds an index to this column.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("username", "users")
    ///     .indexed();
    ///
    /// assert!(col.is_indexed());
    /// ```
    pub fn indexed(mut self) -> Self {
        self.indexed = true;
        self
    }

    /// Returns the name of this column.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("username", "users");
    /// assert_eq!(col.name(), "username");
    /// ```
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns a reference to the default value, if any.
    ///
    /// # Returns
    ///
    /// - `Some(&T)`: A reference to the default value
    /// - `None`: If no default value is set
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("name", "users")
    ///     .default_value("Anonymous".to_string());
    ///
    /// assert_eq!(col.get_default(), Some(&"Anonymous".to_string()));
    /// ```
    pub fn get_default(&self) -> Option<&T> {
        self.default_value.as_ref()
    }

    /// Returns whether this column allows NULL values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let nullable_col = Column::<String>::new("bio", "users");
    /// let not_null_col = Column::<String>::new("name", "users").not_null();
    ///
    /// assert!(nullable_col.is_nullable());
    /// assert!(!not_null_col.is_nullable());
    /// ```
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Returns whether this column has a UNIQUE constraint.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let unique_col = Column::<String>::new("email", "users").unique();
    /// let normal_col = Column::<String>::new("name", "users");
    ///
    /// assert!(unique_col.is_unique());
    /// assert!(!normal_col.is_unique());
    /// ```
    pub fn is_unique(&self) -> bool {
        self.unique
    }

    /// Returns whether this column is a primary key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let pk_col = Column::<i32>::new("id", "users").primary_key();
    /// let normal_col = Column::<String>::new("name", "users");
    ///
    /// assert!(pk_col.is_primary_key());
    /// assert!(!normal_col.is_primary_key());
    /// ```
    pub fn is_primary_key(&self) -> bool {
        self.primary_key
    }

    /// Returns whether this column has an index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let indexed_col = Column::<String>::new("username", "users").indexed();
    /// let normal_col = Column::<String>::new("name", "users");
    ///
    /// assert!(indexed_col.is_indexed());
    /// assert!(!normal_col.is_indexed());
    /// ```
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }
}

impl<T: Copy> Copy for Column<T> where T: Copy {}

/// A type-erased value that can represent any database column value.
///
/// The `Value` enum provides a way to store and convert between different
/// database value types in a type-safe manner. It's used internally by
/// the row system for database value storage and retrieval.
///
/// # Variants
///
/// - `String(String)`: Text data
/// - `Int(i32)`: 32-bit integer
/// - `Long(i64)`: 64-bit integer
/// - `Float(f64)`: 64-bit floating point number
/// - `Bool(bool)`: Boolean value
/// - `Null`: NULL value
///
/// # Type Conversions
///
/// The `Value` enum implements `From<T>` for all supported types and
/// `TryFrom<Value>` for extracting values back to their original types.
///
/// # Example
///
/// ```rust
/// use lume::schema::Value;
///
/// // Create values from Rust types
/// let string_val: Value = "hello".to_string().into();
/// let int_val: Value = 42.into();
/// let bool_val: Value = true.into();
///
/// // Extract values back to Rust types
/// let extracted_string: Result<String, ()> = string_val.try_into();
/// let extracted_int: Result<i32, ()> = int_val.try_into();
/// let extracted_bool: Result<bool, ()> = bool_val.try_into();
///
/// assert_eq!(extracted_string, Ok("hello".to_string()));
/// assert_eq!(extracted_int, Ok(42));
/// assert_eq!(extracted_bool, Ok(true));
/// ```
#[derive(Clone, Debug)]
pub enum Value {
    /// String/text value
    String(String),
    /// 32-bit integer value
    Int(i32),
    /// 64-bit integer value
    Long(i64),
    /// 64-bit floating point value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// NULL value
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Int(i) => write!(f, "{}", i),
            Value::Long(l) => write!(f, "{}", l),
            Value::Float(val) => write!(f, "{}", *val),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "NULL"),
        }
    }
}

// Implement From for common types
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Long(i)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

// Implement TryFrom for extraction
impl TryFrom<Value> for String {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Long(l) => Ok(l),
            Value::Int(i) => Ok(i as i64), // Allow conversion from i32
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(()),
        }
    }
}

// Better approach: Use a trait for type-safe conversion
pub trait IntoValue {
    fn into_db_value(self) -> Value;
}

impl IntoValue for f32 {
    fn into_db_value(self) -> Value {
        Value::Float(self as f64)
    }
}

impl<T> IntoValue for T
where
    Value: From<T>,
{
    fn into_db_value(self) -> Value {
        Value::from(self)
    }
}

impl<T: IntoValue> IntoValue for Option<T> {
    fn into_db_value(self) -> Value {
        match self {
            Some(v) => v.into_db_value(),
            None => Value::Null,
        }
    }
}

pub fn check_type<T: Any>(value: &T) -> Value {
    if let Some(s) = <dyn Any>::downcast_ref::<String>(value) {
        Value::String(s.clone())
    } else if let Some(i) = <dyn Any>::downcast_ref::<i32>(value) {
        Value::Int(*i)
    } else if let Some(l) = <dyn Any>::downcast_ref::<i64>(value) {
        Value::Long(*l)
    } else if let Some(f) = <dyn Any>::downcast_ref::<f32>(value) {
        Value::Float(*f as f64)
    } else if let Some(f) = <dyn Any>::downcast_ref::<f64>(value) {
        Value::Float(*f)
    } else if let Some(b) = <dyn Any>::downcast_ref::<bool>(value) {
        Value::Bool(*b)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<&str>>(value) {
        opt.map(|s| Value::String(s.to_string()))
            .unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<i32>>(value) {
        opt.map(Value::Int).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<i64>>(value) {
        opt.map(Value::Long).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<f32>>(value) {
        opt.map(|f| Value::Float(f as f64)).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<f64>>(value) {
        opt.map(Value::Float).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<bool>>(value) {
        opt.map(Value::Bool).unwrap_or(Value::Null)
    } else {
        debug_assert!(
            false,
            "Unsupported type in check_type: {}",
            std::any::type_name::<T>()
        );
        Value::Null
    }
}
