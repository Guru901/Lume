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
    /// Whether this column auto-increments (MySQL AUTO_INCREMENT)
    auto_increment: bool,
    /// Optional column comment (MySQL COMMENT)
    comment: Option<&'static str>,
    /// Optional character set (MySQL CHARACTER SET)
    charset: Option<&'static str>,
    /// Optional collation (MySQL COLLATE)
    collate: Option<&'static str>,
    /// Whether column has ON UPDATE CURRENT_TIMESTAMP behavior (MySQL)
    on_update_current_timestamp: bool,
    /// Whether this column is invisible (MySQL 8: INVISIBLE)
    invisible: bool,
    /// Optional CHECK constraint expression (MySQL 8)
    check: Option<&'static str>,
    /// Optional generated column definition (VIRTUAL or STORED)
    generated: Option<GeneratedColumn>,
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
    auto_increment: {},
    comment: {:?},
    charset: {:?},
    collate: {:?},
    on_update_current_timestamp: {},
    invisible: {},
    check: {:?},
    generated: {:?},
    table_name: {}
}}",
            self.name,
            self.default_value,
            self.nullable,
            self.unique,
            self.primary_key,
            self.indexed,
            self.auto_increment,
            self.comment,
            self.charset,
            self.collate,
            self.on_update_current_timestamp,
            self.invisible,
            self.check,
            self.generated,
            self.table_name
        )
    }
}

/// MySQL generated column variants
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GeneratedColumn {
    /// Virtual generated column (not stored)
    Virtual(&'static str),
    /// Stored generated column (persisted)
    Stored(&'static str),
}

impl Display for GeneratedColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratedColumn::Virtual(s) => write!(f, "VIRTUAL {}", s),
            GeneratedColumn::Stored(s) => write!(f, "STORED {}", s),
        }
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
            auto_increment: false,
            comment: None,
            charset: None,
            collate: None,
            on_update_current_timestamp: false,
            invisible: false,
            check: None,
            generated: None,
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
    pub fn default_value<K: Into<T>>(mut self, value: K) -> Self {
        self.default_value = Some(value.into());
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

    /// Enables AUTO_INCREMENT on this column (MySQL).
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
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
        self.on_update_current_timestamp = true;
        self
    }

    /// Marks the column as INVISIBLE (MySQL 8).
    pub fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }

    /// Adds a CHECK constraint expression (MySQL 8).
    pub fn check(mut self, expression: &'static str) -> Self {
        self.check = Some(expression);
        self
    }

    /// Defines this column as a VIRTUAL generated column (MySQL) with the given expression.
    pub fn generated_virtual(mut self, expression: &'static str) -> Self {
        self.generated = Some(GeneratedColumn::Virtual(expression));
        self
    }

    /// Defines this column as a STORED generated column (MySQL) with the given expression.
    pub fn generated_stored(mut self, expression: &'static str) -> Self {
        self.generated = Some(GeneratedColumn::Stored(expression));
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

    /// Returns the name of the table this column belongs to
    ///
    /// # Example
    ///
    /// ```rust
    /// use lume::schema::Column;
    ///
    /// let col = Column::<String>::new("username", "users");
    /// assert_eq!(col.table_name(), "users");
    /// ```
    pub fn table_name(&self) -> &'static str {
        self.table_name
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

    /// Returns whether this column is AUTO_INCREMENT.
    pub fn is_auto_increment(&self) -> bool {
        self.auto_increment
    }

    /// Returns the column comment if set.
    pub fn get_comment(&self) -> Option<&'static str> {
        self.comment
    }

    /// Returns the character set if set.
    pub fn get_charset(&self) -> Option<&'static str> {
        self.charset
    }

    /// Returns the collation if set.
    pub fn get_collate(&self) -> Option<&'static str> {
        self.collate
    }

    /// Returns whether the column has ON UPDATE CURRENT_TIMESTAMP behavior.
    pub fn has_on_update_current_timestamp(&self) -> bool {
        self.on_update_current_timestamp
    }

    /// Returns whether the column is INVISIBLE.
    pub fn is_invisible(&self) -> bool {
        self.invisible
    }

    /// Returns the CHECK constraint expression if set.
    pub fn get_check(&self) -> Option<&'static str> {
        self.check
    }

    /// Returns the generated column definition if set.
    pub fn get_generated(&self) -> Option<GeneratedColumn> {
        self.generated
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
/// - `Int8(i8)`: 8-bit signed integer
/// - `Int16(i16)`: 16-bit signed integer
/// - `Int32(i32)`: 32-bit signed integer
/// - `Int64(i64)`: 64-bit signed integer
/// - `UInt8(u8)`: 8-bit unsigned integer
/// - `UInt16(u16)`: 16-bit unsigned integer
/// - `UInt32(u32)`: 32-bit unsigned integer
/// - `UInt64(u64)`: 64-bit unsigned integer
/// - `Float32(f32)`: 32-bit floating point number
/// - `Float64(f64)`: 64-bit floating point number
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
/// let int_val: Value = 42i32.into();
/// let uint_val: Value = 42u64.into();
/// let bool_val: Value = true.into();
///
/// // Extract values back to Rust types
/// let extracted_string: Result<String, ()> = string_val.try_into();
/// let extracted_int: Result<i32, ()> = int_val.try_into();
/// let extracted_uint: Result<u64, ()> = uint_val.try_into();
/// let extracted_bool: Result<bool, ()> = bool_val.try_into();
///
/// assert_eq!(extracted_string, Ok("hello".to_string()));
/// assert_eq!(extracted_int, Ok(42));
/// assert_eq!(extracted_uint, Ok(42));
/// assert_eq!(extracted_bool, Ok(true));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// String/text value
    String(String),
    /// 8-bit signed integer value
    Int8(i8),
    /// 16-bit signed integer value
    Int16(i16),
    /// 32-bit signed integer value
    Int32(i32),
    /// 64-bit signed integer value
    Int64(i64),
    /// 8-bit unsigned integer value
    UInt8(u8),
    /// 16-bit unsigned integer value
    UInt16(u16),
    /// 32-bit unsigned integer value
    UInt32(u32),
    /// 64-bit unsigned integer value
    UInt64(u64),
    /// 32-bit floating point value
    Float32(f32),
    /// 64-bit floating point value
    Float64(f64),
    /// Boolean value
    Bool(bool),

    Array(Vec<Value>),
    /// NULL value
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Int8(i) => write!(f, "{}", i),
            Value::Int16(i) => write!(f, "{}", i),
            Value::Int32(i) => write!(f, "{}", i),
            Value::Int64(i) => write!(f, "{}", i),
            Value::UInt8(u) => write!(f, "{}", u),
            Value::UInt16(u) => write!(f, "{}", u),
            Value::UInt32(u) => write!(f, "{}", u),
            Value::UInt64(u) => write!(f, "{}", u),
            Value::Float32(val) => write!(f, "{}", val),
            Value::Float64(val) => write!(f, "{}", val),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(arr) => write!(f, "{:?}", arr),
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

// Signed integer types
impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Value::Int8(i)
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Value::Int16(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int32(i)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int64(i)
    }
}

// Unsigned integer types
impl From<u8> for Value {
    fn from(u: u8) -> Self {
        Value::UInt8(u)
    }
}

impl From<u16> for Value {
    fn from(u: u16) -> Self {
        Value::UInt16(u)
    }
}

impl From<u32> for Value {
    fn from(u: u32) -> Self {
        Value::UInt32(u)
    }
}

impl From<u64> for Value {
    fn from(u: u64) -> Self {
        Value::UInt64(u)
    }
}

// Floating point types
impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float32(f)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float64(f)
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

// Signed integer TryFrom implementations
impl TryFrom<Value> for i8 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) => Ok(i),
            Value::Int16(i) if i >= i8::MIN as i16 && i <= i8::MAX as i16 => Ok(i as i8),
            Value::Int32(i) if i >= i8::MIN as i32 && i <= i8::MAX as i32 => Ok(i as i8),
            Value::Int64(i) if i >= i8::MIN as i64 && i <= i8::MAX as i64 => Ok(i as i8),
            Value::UInt8(u) if u <= i8::MAX as u8 => Ok(u as i8),
            Value::UInt16(u) if u <= i8::MAX as u16 => Ok(u as i8),
            Value::UInt32(u) if u <= i8::MAX as u32 => Ok(u as i8),
            Value::UInt64(u) if u <= i8::MAX as u64 => Ok(u as i8),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for i16 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) => Ok(i as i16),
            Value::Int16(i) => Ok(i),
            Value::Int32(i) if i >= i16::MIN as i32 && i <= i16::MAX as i32 => Ok(i as i16),
            Value::Int64(i) if i >= i16::MIN as i64 && i <= i16::MAX as i64 => Ok(i as i16),
            Value::UInt8(u) => Ok(u as i16),
            Value::UInt16(u) if u <= i16::MAX as u16 => Ok(u as i16),
            Value::UInt32(u) if u <= i16::MAX as u32 => Ok(u as i16),
            Value::UInt64(u) if u <= i16::MAX as u64 => Ok(u as i16),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) => Ok(i as i32),
            Value::Int16(i) => Ok(i as i32),
            Value::Int32(i) => Ok(i),
            Value::Int64(i) if i >= i32::MIN as i64 && i <= i32::MAX as i64 => Ok(i as i32),
            Value::UInt8(u) => Ok(u as i32),
            Value::UInt16(u) => Ok(u as i32),
            Value::UInt32(u) if u <= i32::MAX as u32 => Ok(u as i32),
            Value::UInt64(u) if u <= i32::MAX as u64 => Ok(u as i32),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) => Ok(i as i64),
            Value::Int16(i) => Ok(i as i64),
            Value::Int32(i) => Ok(i as i64),
            Value::Int64(i) => Ok(i),
            Value::UInt8(u) => Ok(u as i64),
            Value::UInt16(u) => Ok(u as i64),
            Value::UInt32(u) => Ok(u as i64),
            Value::UInt64(u) if u <= i64::MAX as u64 => Ok(u as i64),
            _ => Err(()),
        }
    }
}

// Unsigned integer TryFrom implementations
impl TryFrom<Value> for u8 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) if i >= 0 => Ok(i as u8),
            Value::Int16(i) if i >= 0 && i <= u8::MAX as i16 => Ok(i as u8),
            Value::Int32(i) if i >= 0 && i <= u8::MAX as i32 => Ok(i as u8),
            Value::Int64(i) if i >= 0 && i <= u8::MAX as i64 => Ok(i as u8),
            Value::UInt8(u) => Ok(u),
            Value::UInt16(u) if u <= u8::MAX as u16 => Ok(u as u8),
            Value::UInt32(u) if u <= u8::MAX as u32 => Ok(u as u8),
            Value::UInt64(u) if u <= u8::MAX as u64 => Ok(u as u8),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for u16 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) if i >= 0 => Ok(i as u16),
            Value::Int16(i) if i >= 0 => Ok(i as u16),
            Value::Int32(i) if i >= 0 && i <= u16::MAX as i32 => Ok(i as u16),
            Value::Int64(i) if i >= 0 && i <= u16::MAX as i64 => Ok(i as u16),
            Value::UInt8(u) => Ok(u as u16),
            Value::UInt16(u) => Ok(u),
            Value::UInt32(u) if u <= u16::MAX as u32 => Ok(u as u16),
            Value::UInt64(u) if u <= u16::MAX as u64 => Ok(u as u16),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for u32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) if i >= 0 => Ok(i as u32),
            Value::Int16(i) if i >= 0 => Ok(i as u32),
            Value::Int32(i) if i >= 0 => Ok(i as u32),
            Value::Int64(i) if i >= 0 && i <= u32::MAX as i64 => Ok(i as u32),
            Value::UInt8(u) => Ok(u as u32),
            Value::UInt16(u) => Ok(u as u32),
            Value::UInt32(u) => Ok(u),
            Value::UInt64(u) if u <= u32::MAX as u64 => Ok(u as u32),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for u64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int8(i) if i >= 0 => Ok(i as u64),
            Value::Int16(i) if i >= 0 => Ok(i as u64),
            Value::Int32(i) if i >= 0 => Ok(i as u64),
            Value::Int64(i) if i >= 0 => Ok(i as u64),
            Value::UInt8(u) => Ok(u as u64),
            Value::UInt16(u) => Ok(u as u64),
            Value::UInt32(u) => Ok(u as u64),
            Value::UInt64(u) => Ok(u),
            _ => Err(()),
        }
    }
}

// Floating point TryFrom implementations
impl TryFrom<Value> for f32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Float32(f) => Ok(f),
            Value::Float64(f) if f >= f32::MIN as f64 && f <= f32::MAX as f64 => Ok(f as f32),
            Value::Int8(i) => Ok(i as f32),
            Value::Int16(i) => Ok(i as f32),
            Value::Int32(i) => Ok(i as f32),
            Value::Int64(i) => Ok(i as f32),
            Value::UInt8(u) => Ok(u as f32),
            Value::UInt16(u) => Ok(u as f32),
            Value::UInt32(u) => Ok(u as f32),
            Value::UInt64(u) => Ok(u as f32),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Float32(f) => Ok(f as f64),
            Value::Float64(f) => Ok(f),
            Value::Int8(i) => Ok(i as f64),
            Value::Int16(i) => Ok(i as f64),
            Value::Int32(i) => Ok(i as f64),
            Value::Int64(i) => Ok(i as f64),
            Value::UInt8(u) => Ok(u as f64),
            Value::UInt16(u) => Ok(u as f64),
            Value::UInt32(u) => Ok(u as f64),
            Value::UInt64(u) => Ok(u as f64),
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

/// Converts a reference to a value of any supported type into a [`Value`] enum.
///
/// This function attempts to downcast the provided reference to a known supported type
/// (`String`, `i32`, `i64`, `f32`, `f64`, `bool`, or their `Option` variants) and
/// returns the corresponding [`Value`] variant. If the type is not supported, it
/// returns [`Value::Null`] and triggers a debug assertion failure in debug builds.
///
/// # Type Parameters
///
/// - `T`: The type of the value, which must implement [`Any`].
///
/// # Arguments
///
/// * `value` - A reference to the value to convert.
///
/// # Returns
///
/// A [`Value`] representing the input value, or [`Value::Null`] if the type is unsupported.
///
/// # Panics
///
/// This function will trigger a debug assertion failure if called with an unsupported type.
///
/// # Examples
///
/// ```
/// use lume::schema::{convert_to_value, Value};
///
/// let int_val = 42i32;
/// assert_eq!(convert_to_value(&int_val), Value::Int32(42));
///
/// let float_val = 3.14f64;
/// assert_eq!(convert_to_value(&float_val), Value::Float64(3.14));
///
/// let opt_str: Option<&str> = Some("hello");
/// assert_eq!(convert_to_value(&opt_str), Value::String("hello".to_string()));
///
/// let none_val: Option<i32> = None;
/// assert_eq!(convert_to_value(&none_val), Value::Null);
/// ```
pub fn convert_to_value<T: Any>(value: &T) -> Value {
    if let Some(s) = <dyn Any>::downcast_ref::<String>(value) {
        Value::String(s.clone())
    } else if let Some(s) = <dyn Any>::downcast_ref::<&str>(value) {
        Value::String((*s).to_string())
    } else if let Some(i) = <dyn Any>::downcast_ref::<i8>(value) {
        Value::Int8(*i)
    } else if let Some(i) = <dyn Any>::downcast_ref::<i16>(value) {
        Value::Int16(*i)
    } else if let Some(i) = <dyn Any>::downcast_ref::<i32>(value) {
        Value::Int32(*i)
    } else if let Some(i) = <dyn Any>::downcast_ref::<i64>(value) {
        Value::Int64(*i)
    } else if let Some(u) = <dyn Any>::downcast_ref::<u8>(value) {
        Value::UInt8(*u)
    } else if let Some(u) = <dyn Any>::downcast_ref::<u16>(value) {
        Value::UInt16(*u)
    } else if let Some(u) = <dyn Any>::downcast_ref::<u32>(value) {
        Value::UInt32(*u)
    } else if let Some(u) = <dyn Any>::downcast_ref::<u64>(value) {
        Value::UInt64(*u)
    } else if let Some(f) = <dyn Any>::downcast_ref::<f32>(value) {
        Value::Float32(*f)
    } else if let Some(f) = <dyn Any>::downcast_ref::<f64>(value) {
        Value::Float64(*f)
    } else if let Some(b) = <dyn Any>::downcast_ref::<bool>(value) {
        Value::Bool(*b)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<&str>>(value) {
        opt.map(|s| Value::String(s.to_string()))
            .unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<String>>(value) {
        opt.as_ref()
            .map(|s| Value::String(s.clone()))
            .unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<&String>>(value) {
        match opt {
            Some(s) => Value::String((*s).clone()),
            None => Value::Null,
        }
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<i8>>(value) {
        opt.map(Value::Int8).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<i16>>(value) {
        opt.map(Value::Int16).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<i32>>(value) {
        opt.map(Value::Int32).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<i64>>(value) {
        opt.map(Value::Int64).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<u8>>(value) {
        opt.map(Value::UInt8).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<u16>>(value) {
        opt.map(Value::UInt16).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<u32>>(value) {
        opt.map(Value::UInt32).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<u64>>(value) {
        opt.map(Value::UInt64).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<f32>>(value) {
        opt.map(Value::Float32).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<f64>>(value) {
        opt.map(Value::Float64).unwrap_or(Value::Null)
    } else if let Some(opt) = <dyn Any>::downcast_ref::<Option<bool>>(value) {
        opt.map(Value::Bool).unwrap_or(Value::Null)
    } else {
        debug_assert!(
            false,
            "Unsupported type in convert_to_value: {}",
            std::any::type_name::<T>()
        );
        Value::Null
    }
}
