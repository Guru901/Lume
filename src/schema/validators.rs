/// Represents different types of validators that can be applied to a column.
///
/// Validators provide semantic hints for validation and UI generation,
/// helping ensure data integrity and proper input validation at the application layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnValidators {
    /// Validates that the value is a properly formatted email address.
    Email,
    /// Validates that the value is a valid URL/link.
    Url,
    /// Validates that string length is at least the specified minimum.
    MinLen(usize),
    /// Validates that string length does not exceed the specified maximum.
    MaxLen(usize),
    /// Validates that numeric value is at least the specified minimum.
    Min(usize),
    /// Validates that numeric value does not exceed the specified maximum.
    Max(usize),
    /// Validates that the value matches the specified regex pattern.
    Pattern(&'static str),
}
