/// Error type for database operations.
///
/// This error type wraps SQLx errors and provides a unified error interface
/// for all database operations in Lume.
///
/// # Example
///
/// ```no_run
/// use lume::database::{Database, error::DatabaseError};
///
/// async fn example() -> Result<(), DatabaseError> {
///     let db = Database::connect("mysql://invalid_url").await?;
///     // Handle database operations...
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct DatabaseError(sqlx::Error);

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database error: {}", self.0)
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        DatabaseError(err)
    }
}
