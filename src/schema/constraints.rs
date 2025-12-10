use std::fmt::Display;

/// Represents different types of constraints that can be applied to a column.
///
/// Constraints define database-level rules and behaviors for a column,
/// such as nullability, uniqueness, indexing, and auto-increment behavior.
#[derive(Clone, Debug, PartialEq)]
pub enum ColumnConstraint {
    /// Column cannot contain NULL values (NOT NULL constraint).
    NonNullable,
    /// Column values must be unique across all rows (UNIQUE constraint).
    Unique,
    /// Column is the primary key for the table.
    PrimaryKey,
    /// Column has an index created for faster lookups.
    Indexed,
    /// Column value is automatically incremented (AUTO_INCREMENT in MySQL).
    AutoIncrement,
    /// Column is hidden from SELECT * queries (MySQL 8+ INVISIBLE).
    Invisible,
    /// Column is automatically updated to current timestamp on row update.
    OnUpdateCurrentTimestamp,
    /// Column has a CHECK constraint with the specified expression.
    Check(&'static str),
    /// Column is a generated column (VIRTUAL or STORED).
    Generated(GeneratedColumn),
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
