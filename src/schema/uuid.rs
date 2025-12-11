#![warn(missing_docs)]

//! UUID type support for schema definitions.

use std::fmt::Display;

use crate::schema::Value;

/// A UUID type wrapper for use in schema definitions.
///
/// This type maps to:
/// - `UUID` in PostgreSQL
/// - `CHAR(36)` in MySQL/SQLite
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
/// use lume::schema::Uuid;
///
/// define_schema! {
///     User {
///         id: Uuid [primary_key().not_null().default_random()],
///         name: String [not_null()],
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Uuid(pub String);

impl Uuid {
    /// Creates a new UUID from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Uuid(s.into())
    }

    /// Returns the UUID as a string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the UUID and returns the inner string.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Uuid {
    fn from(s: String) -> Self {
        Uuid(s)
    }
}

impl From<&str> for Uuid {
    fn from(s: &str) -> Self {
        Uuid(s.to_string())
    }
}

impl From<Uuid> for String {
    fn from(uuid: Uuid) -> Self {
        uuid.0
    }
}

impl From<Uuid> for Value {
    fn from(uuid: Uuid) -> Self {
        Value::Uuid(uuid)
    }
}

impl From<&Uuid> for Value {
    fn from(uuid: &Uuid) -> Self {
        Value::Uuid(uuid.clone())
    }
}

impl TryFrom<Value> for Uuid {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Uuid(uuid) => Ok(uuid),
            Value::String(s) => Ok(Uuid(s)),
            _ => Err(()),
        }
    }
}
