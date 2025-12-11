use std::any::Any;
use std::fmt::{Debug, Display};

use crate::schema::Uuid;

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

    Uuid(Uuid),
    /// 8-bit signed integer value
    Int8(i8),
    /// 16-bit signed integer value
    Int16(i16),
    /// 32-bit signed integer value
    Int32(i32),
    /// 64-bit signed integer value
    Int64(i64),
    /// 8-bit unsigned integer value
    #[cfg(any(feature = "mysql", feature = "sqlite"))]
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

    /// Array value containing a vector of `Value` elements.
    Array(Vec<Value>),

    /// NULL value
    Null,

    /// Between value
    Between(Box<Value>, Box<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Int8(i) => write!(f, "{}", i),
            Value::Int16(i) => write!(f, "{}", i),
            Value::Int32(i) => write!(f, "{}", i),
            Value::Int64(i) => write!(f, "{}", i),
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
            Value::UInt8(u) => write!(f, "{}", u),
            Value::UInt16(u) => write!(f, "{}", u),
            Value::UInt32(u) => write!(f, "{}", u),
            Value::UInt64(u) => write!(f, "{}", u),
            Value::Float32(val) => write!(f, "{}", val),
            Value::Float64(val) => write!(f, "{}", val),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(arr) => write!(f, "{:?}", arr),
            Value::Between(min, max) => write!(f, "BETWEEN {} AND {}", min, max),
            Value::Null => write!(f, "NULL"),
            Value::Uuid(uuid) => write!(f, "{}", uuid),
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
#[cfg(any(feature = "mysql", feature = "sqlite"))]
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

impl TryFrom<Value> for Vec<String> {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for v in arr {
                    match v {
                        Value::String(s) => out.push(s),
                        _ => return Err(()),
                    }
                }
                Ok(out)
            }
            _ => Err(()),
        }
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
            Value::UInt8(u) => Ok(u as f32),
            Value::UInt16(u) => Ok(u as f32),
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
            Value::UInt32(u) => Ok(u as f32),
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
            #[cfg(any(feature = "mysql", feature = "sqlite"))]
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
pub fn convert_to_value<T: Any + Debug>(value: &T) -> Value {
    if let Some(uuid) = <dyn Any>::downcast_ref::<crate::schema::Uuid>(value) {
        Value::String(uuid.as_str().to_string())
    } else if let Some(s) = <dyn Any>::downcast_ref::<String>(value) {
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
        #[cfg(any(feature = "mysql", feature = "sqlite"))]
        return Value::UInt8(*u);

        #[cfg(feature = "postgres")]
        return Value::Int16(*u as i16);
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
        #[cfg(any(feature = "mysql", feature = "sqlite"))]
        return opt.map(Value::UInt8).unwrap_or(Value::Null);

        #[cfg(feature = "postgres")]
        return opt.map(|u| Value::Int16(u as i16)).unwrap_or(Value::Null);
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
        // Fallback to Debug
        let dbg = value as &dyn std::fmt::Debug;
        let s = format!("{:?}", dbg);
        if s == "None" {
            Value::Null
        } else {
            Value::String(s)
        }
    }
}
