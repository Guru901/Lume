/// Represents errors that can occur during database operations in Lume.
///
/// This type wraps underlying SQLx errors as well as Lume-specific error cases,
/// providing a unified error type for all database-related fallible operations.
///
/// # Variants
///
/// - [`InvalidValue(String)`]: An invalid value was provided to a database operation.
/// - [`ConnectionError(sqlx::Error)`]: An error occurred while establishing a database connection.
/// - [`QueryError(String)`]: An error occurred during query preparation or execution.
/// - [`ExecutionError(String)`]: An error occurred while executing a database operation.
///
/// # Examples
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
///
/// Handling errors:
///
/// ```no_run
/// use lume::database::{Database, error::DatabaseError};
///
/// async fn do_db() {
///     match Database::connect("mysql://bad_url").await {
///         Ok(db) => { /* proceed */ }
///         Err(e) => eprintln!("Database error: {}", e.reason()),
///     }
/// }
/// ```
#[derive(Debug)]
pub enum DatabaseError {
    /// An invalid value was provided to a database operation
    InvalidValue(String),
    /// A connection-related error from the underlying driver
    ConnectionError(sqlx::Error),
    /// An error during query preparation or execution
    QueryError(String),
    /// An error in the execution of a database operation
    ExecutionError(String),
}

impl DatabaseError {
    /// Returns a human-readable reason for the error.
    pub fn reason(&self) -> String {
        match self {
            DatabaseError::InvalidValue(reason) => reason.clone(),
            DatabaseError::ConnectionError(e) => e.to_string(),
            DatabaseError::QueryError(e) => e.clone(),
            DatabaseError::ExecutionError(e) => e.clone(),
        }
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason())
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DatabaseError::ConnectionError(e) => Some(e),
            _ => None,
        }
    }
}
