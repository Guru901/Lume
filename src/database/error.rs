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
pub enum DatabaseError {
    InvalidValue(String),
    ConnectionError(sqlx::Error),
    QueryError(String),
    ExecutionError(String),
}

impl DatabaseError {
    pub fn reason(&self) -> String {
        match self {
            DatabaseError::InvalidValue(reason) => reason.clone(),
            DatabaseError::ConnectionError(e) => e.to_string(),
            DatabaseError::QueryError(e) => e.clone(),
            DatabaseError::ExecutionError(e) => e.clone(),
        }
    }
}
