use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

// Generic Column that knows its Rust type at compile time
#[derive(Clone, Debug)]
pub struct Column<T> {
    pub(crate) name: &'static str,
    default_value: Option<T>,
    table_name: &'static str,
    nullable: bool,
    unique: bool,
    primary_key: bool,
    indexed: bool,
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

    // Can't be const because we're storing T
    pub fn default_value(mut self, value: T) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false; // Primary keys are always not null
        self
    }

    pub fn indexed(mut self) -> Self {
        self.indexed = true;
        self
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn get_default(&self) -> Option<&T> {
        self.default_value.as_ref()
    }

    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    pub fn is_unique(&self) -> bool {
        self.unique
    }

    pub fn is_primary_key(&self) -> bool {
        self.primary_key
    }

    pub fn is_indexed(&self) -> bool {
        self.indexed
    }
}

impl<T: Copy> Copy for Column<T> where T: Copy {}

// Value enum for storage
#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    Null,
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

impl TryFrom<Value> for bool {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(()),
        }
    }
}
