#![warn(missing_docs)]

//! # Insert Operation
//!
//! This module provides the [`Insert`] struct for type-safe insertion of records
//! into a MySQL, PostgreSQL, or SQLite database using a schema definition. It supports optional
//! returning of inserted rows and handles value binding for various SQL types.

use crate::database::error::DatabaseError;
use crate::dialects::get_dialect;
use crate::helpers::{StartingSql, bind_column_value, get_starting_sql, validate_column_value};
use crate::row::Row;
use crate::schema::{ColumnConstraint, ColumnInfo, Schema, Select, Value};

#[cfg(feature = "mysql")]
use sqlx::MySqlPool;

#[cfg(feature = "postgres")]
use sqlx::PgPool;

#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;

use std::collections::HashMap;
use std::fmt::Debug;
#[cfg(any(feature = "sqlite", feature = "mysql", feature = "postgres"))]
use std::sync::Arc;

/// Select columns that should be included in an INSERT statement based on provided values.
///
/// Omits columns that have defaults or are auto-incremented when their value is absent or Null.
fn select_insertable_columns<'a>(
    all_columns: Vec<ColumnInfo<'a>>,
    values: &HashMap<String, Value>,
) -> Vec<ColumnInfo<'a>> {
    all_columns
        .into_iter()
        .filter(|col| match values.get(col.name) {
            None => {
                !(col.has_default || col.constraints.contains(&ColumnConstraint::AutoIncrement))
            }
            Some(Value::Null) => {
                !(col.has_default || col.constraints.contains(&ColumnConstraint::AutoIncrement))
            }
            _ => true,
        })
        .collect()
}

/// A type-safe insert operation for a given schema type.
///
/// The [`Insert`] struct allows you to insert a record of type `T` (which must
/// implement [`Schema`] and [`Debug`]) into the corresponding database table.
/// It builds the SQL statement and binds values according to the schema.
///
/// # Example
///
/// ```no_run
/// use lume::database::Database;
/// use lume::define_schema;
/// use lume::schema::Schema;
/// use lume::schema::ColumnInfo;
///
/// define_schema! {
///     User {
///         id: i32 [primary_key()],
///         name: String [not_null()],
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let db = Database::connect("mysql://...").await.unwrap();
///     db.insert(User { id: 1, name: "guru".to_string() })
///         .execute()
///         .await
///         .unwrap();
/// }
/// ```
pub struct Insert<T> {
    /// The data to be inserted.
    data: T,

    #[cfg(feature = "mysql")]
    /// The database connection pool.
    conn: Arc<MySqlPool>,

    #[cfg(feature = "postgres")]
    /// The database connection pool.
    conn: Arc<PgPool>,

    #[cfg(feature = "sqlite")]
    /// The database connection pool.
    conn: Arc<SqlitePool>,

    /// Whether to return the inserted row(s).
    returning: Vec<&'static str>,
}

impl<T: Schema + Debug> Insert<T> {
    #[cfg(feature = "mysql")]
    /// Creates a new [`Insert`] operation for the given data and connection.
    pub fn new(data: T, conn: Arc<MySqlPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    #[cfg(feature = "postgres")]
    /// Creates a new [`Insert`] operation for the given data and connection.
    pub fn new(data: T, conn: Arc<PgPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    #[cfg(feature = "sqlite")]
    /// Creates a new [`Insert`] operation for the given data and connection.
    pub fn new(data: T, conn: Arc<SqlitePool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    /// Configures the insert to return the inserted row(s).
    ///
    /// # Returns
    ///
    /// The [`Insert`] instance with returning enabled.
    pub fn returning<S: Select + Debug>(mut self, select: S) -> Self {
        self.returning = select.get_selected();
        self
    }

    /// Executes the insert operation asynchronously.
    ///
    /// This method builds the SQL `INSERT` statement, binds all values
    /// according to the schema, and executes the query.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the insert was successful.
    /// * `Err(DatabaseError)` if an error occurred.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use lume::database::Database;
    /// # use lume::define_schema;
    /// # use lume::schema::Schema;
    /// # use lume::schema::ColumnInfo;
    /// # define_schema! {
    /// #     User {
    /// #         id: i32 [primary_key()],
    /// #         name: String [not_null()],
    /// #     }
    /// # }
    /// # #[tokio::main]
    /// # async fn main() {
    /// let db = Database::connect("mysql://...").await.unwrap();
    /// db.insert(User { id: 1, name: "guru".to_string() })
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn execute(self) -> Result<Option<Vec<Row<T>>>, DatabaseError> {
        let conn = self.conn.acquire().await;

        if let Err(e) = conn {
            return Err(DatabaseError::ConnectionError(e));
        }

        let mut conn = conn.unwrap();

        let values = self.data.values();
        let all_columns = T::get_all_columns();

        // Select columns to include: omit columns with defaults/auto_increment when value is None/Null
        let selected: Vec<ColumnInfo> = select_insertable_columns(all_columns, &values);

        let sql = get_starting_sql(StartingSql::Insert, T::table_name());
        let sql = get_dialect().insert_sql(sql, &selected);
        let mut query = sqlx::query(&sql);

        for col in selected.iter() {
            let value = values.get(col.name);
            if validate_column_value(col, value) {
                query = bind_column_value(query, col, value);
            } else {
                eprintln!("Warning: Column {} is not valid for insert", col.name);
                return Err(DatabaseError::InvalidValue(format!(
                    "Column {} is not valid for insert",
                    col.name
                )));
            }
        }

        // For PostgreSQL with RETURNING, we need to add RETURNING clause to the INSERT
        #[cfg(feature = "postgres")]
        if !self.returning.is_empty() {
            let sql = get_starting_sql(StartingSql::Insert, T::table_name());
            let sql = get_dialect().insert_sql(sql, &selected);
            let sql = get_dialect().returning_sql(sql, &self.returning);
            let mut query = sqlx::query(&sql);

            for col in selected.iter() {
                let value = values.get(col.name);
                query = bind_column_value(query, col, value);
            }

            let rows = query.fetch_all(&mut *conn).await;
            if let Err(e) = rows {
                return Err(DatabaseError::QueryError(e.to_string()));
            }
            let rows = rows.unwrap();
            let rows = Row::<T>::from_postgres_row(rows, None);
            return Ok(Some(rows));
        }

        // For SQLite with "RETURNING"
        #[cfg(feature = "sqlite")]
        if !self.returning.is_empty() {
            let sql = get_starting_sql(StartingSql::Insert, T::table_name());
            let sql = get_dialect().insert_sql(sql, &selected);
            let sql = get_dialect().returning_sql(sql, &self.returning);
            let mut query = sqlx::query(&sql);

            for col in selected.iter() {
                let value = values.get(col.name);
                query = bind_column_value(query, col, value);
            }

            let rows = query.fetch_all(&mut *conn).await;
            if let Err(e) = rows {
                return Err(DatabaseError::QueryError(e.to_string()));
            }
            let rows = rows.unwrap();
            let rows = Row::<T>::from_sqlite_row(rows, None);
            return Ok(Some(rows));
        }

        let _result = query.execute(&mut *conn).await;

        if let Err(e) = _result {
            return Err(DatabaseError::ExecutionError(e.to_string()));
        }

        let _result = _result.unwrap();

        if self.returning.is_empty() {
            return Ok(None);
        }

        // For MySQL, build SELECT ... WHERE id = ? using either provided id or last_insert_id
        #[cfg(feature = "mysql")]
        {
            let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
            let mut select_sql = get_dialect().returning_sql(select_sql, &self.returning);
            select_sql.push_str(format!(" FROM {} WHERE id = ?;", T::table_name()).as_str());

            let conn = self.conn.acquire().await;

            if let Err(e) = conn {
                return Err(DatabaseError::ConnectionError(e));
            }

            let mut conn = conn.unwrap();

            let mut query = sqlx::query(&select_sql);

            query = query.bind(_result.last_insert_id());

            let rows = query.fetch_all(&mut *conn).await;

            if let Err(e) = rows {
                return Err(DatabaseError::QueryError(e.to_string()));
            }

            let rows = rows.unwrap();

            let rows = Row::<T>::from_mysql_row(rows, None);
            Ok(Some(rows))
        }

        #[cfg(feature = "sqlite")]
        {
            // In SQLite, if user called returning(), they already got results above.
            // Otherwise, emulate by SELECT ... WHERE rowid = last_insert_rowid().

            if self.returning.is_empty() {
                return Ok(None);
            }

            let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
            let mut select_sql = get_dialect().returning_sql(select_sql, &self.returning);

            // Look for an "id" column in the table schema, not just the values
            let has_id_column = {
                let columns = T::get_all_columns();
                columns.iter().any(|col| col.name == "id")
            };

            let id_col = if has_id_column { "id" } else { "rowid" };
            select_sql.push_str(&format!(
                " FROM {} WHERE {} = last_insert_rowid();",
                T::table_name(),
                id_col
            ));

            let query = sqlx::query(&select_sql);

            let rows = query.fetch_all(&mut *conn).await;

            if let Err(e) = rows {
                return Err(DatabaseError::QueryError(e.to_string()));
            }

            let rows = rows.unwrap();

            let rows = Row::<T>::from_sqlite_row(rows, None);
            Ok(Some(rows))
        }

        #[cfg(feature = "postgres")]
        {
            // This should not be reached as we handle RETURNING above
            Ok(None)
        }
    }
}

/// A type-safe insert operation for inserting multiple records of a given schema type.
///
/// Executes one INSERT per record for simplicity and correctness. This can be
/// optimized later to a multi-row VALUES statement if needed.
pub struct InsertMany<T: Schema + Debug> {
    /// The list of records to be inserted.
    data: Vec<T>,

    #[cfg(feature = "mysql")]
    /// The database connection pool.
    conn: Arc<MySqlPool>,

    #[cfg(feature = "postgres")]
    /// The database connection pool.
    conn: Arc<PgPool>,

    #[cfg(feature = "sqlite")]
    /// The database connection pool.
    conn: Arc<SqlitePool>,

    /// Whether to return the inserted rows.
    returning: Vec<&'static str>,
}

impl<T: Schema + Debug> InsertMany<T> {
    #[cfg(feature = "mysql")]
    /// Creates a new [`InsertMany`] operation for the given records and connection.
    pub fn new(data: Vec<T>, conn: Arc<MySqlPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    #[cfg(feature = "postgres")]
    /// Creates a new [`InsertMany`] operation for the given records and connection.
    pub fn new(data: Vec<T>, conn: Arc<PgPool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    #[cfg(feature = "sqlite")]
    /// Creates a new [`InsertMany`] operation for the given records and connection.
    pub fn new(data: Vec<T>, conn: Arc<SqlitePool>) -> Self {
        Self {
            data,
            conn,
            returning: Vec::new(),
        }
    }

    /// Configures the insert to return the inserted row(s).
    pub fn returning<S: Select + Debug>(mut self, select: S) -> Self {
        self.returning = select.get_selected();
        self
    }

    /// Executes the insert operation for all records asynchronously.
    pub async fn execute(self) -> Result<Option<Vec<Row<T>>>, DatabaseError> {
        let conn = self.conn.acquire().await;

        if let Err(e) = conn {
            return Err(DatabaseError::ConnectionError(e));
        }

        let mut conn = conn.unwrap();

        let mut final_rows = Vec::new();
        let mut inserted_ids: Vec<u64> = Vec::new();

        for record in &self.data {
            let values = record.values();
            let all_columns = T::get_all_columns();
            let selected: Vec<ColumnInfo> = select_insertable_columns(all_columns, &values);

            let sql = get_starting_sql(StartingSql::Insert, T::table_name());
            let sql = get_dialect().insert_sql(sql, &selected);
            let mut query = sqlx::query(&sql);

            for col in selected.iter() {
                let value = values.get(col.name);
                query = bind_column_value(query, col, value);
            }

            #[cfg(feature = "mysql")]
            {
                let result = query.execute(&mut *conn).await;

                if let Err(e) = result {
                    return Err(DatabaseError::ExecutionError(e.to_string()));
                }

                let result = result.unwrap();

                // Capture id: prefer provided id, else last_insert_id
                if let Some(id_val) = values.get("id") {
                    match id_val {
                        Value::Array(_) => inserted_ids.push(result.last_insert_id()),
                        Value::Int64(v) => inserted_ids.push(*v as u64),
                        Value::Int32(v) => inserted_ids.push(*v as u64),
                        Value::Int16(v) => inserted_ids.push(*v as u64),
                        Value::Int8(v) => inserted_ids.push(*v as u64),
                        Value::UInt64(v) => inserted_ids.push(*v),
                        Value::UInt32(v) => inserted_ids.push(*v as u64),
                        Value::UInt16(v) => inserted_ids.push(*v as u64),
                        Value::UInt8(v) => inserted_ids.push(*v as u64),
                        Value::String(_)
                        | Value::Float32(_)
                        | Value::Float64(_)
                        | Value::Bool(_)
                        | Value::Between(_, _)
                        | Value::Null => inserted_ids.push(result.last_insert_id()),
                    }
                } else {
                    inserted_ids.push(result.last_insert_id());
                }
            }

            #[cfg(feature = "postgres")]
            {
                // For PostgreSQL, if returning is requested, we need to use RETURNING clause
                if !self.returning.is_empty() {
                    let sql = get_starting_sql(StartingSql::Insert, T::table_name());
                    let sql = get_dialect().insert_sql(sql, &selected);
                    let sql = get_dialect().returning_sql(sql, &self.returning);
                    let mut query = sqlx::query(&sql);

                    for col in selected.iter() {
                        let value = values.get(col.name);
                        query = bind_column_value(query, col, value);
                    }

                    let rows = query.fetch_all(&mut *conn).await;
                    if let Err(e) = rows {
                        return Err(DatabaseError::QueryError(e.to_string()));
                    }
                    let rows = rows.unwrap();
                    let rows = Row::<T>::from_postgres_row(rows, None);
                    final_rows.extend(rows);
                } else {
                    // Execute without returning
                    match query.execute(&mut *conn).await {
                        Ok(_) => {}
                        Err(e) => return Err(DatabaseError::ExecutionError(e.to_string())),
                    }

                    // Capture id: prefer provided id
                    if let Some(id_val) = values.get("id") {
                        match id_val {
                            Value::Int64(v) => inserted_ids.push(*v as u64),
                            Value::Int32(v) => inserted_ids.push(*v as u64),
                            Value::Int16(v) => inserted_ids.push(*v as u64),
                            Value::Int8(v) => inserted_ids.push(*v as u64),
                            _ => {
                                // For PostgreSQL without RETURNING, we can't get the id
                                // This is a limitation - user should use returning() to get ids
                            }
                        }
                    }
                }
            }

            #[cfg(feature = "sqlite")]
            {
                // For SQLite, if returning is requested, we use RETURNING clause
                if !self.returning.is_empty() {
                    let sql = get_starting_sql(StartingSql::Insert, T::table_name());
                    let sql = get_dialect().insert_sql(sql, &selected);
                    let sql = get_dialect().returning_sql(sql, &self.returning);
                    let mut query = sqlx::query(&sql);

                    for col in selected.iter() {
                        let value = values.get(col.name);
                        query = bind_column_value(query, col, value);
                    }

                    let rows = query.fetch_all(&mut *conn).await;
                    if let Err(e) = rows {
                        return Err(DatabaseError::QueryError(e.to_string()));
                    }
                    let rows = rows.unwrap();
                    let rows = Row::<T>::from_sqlite_row(rows, None);
                    final_rows.extend(rows);
                } else {
                    // Execute without returning
                    let result = query.execute(&mut *conn).await;
                    if let Err(e) = result {
                        return Err(DatabaseError::ExecutionError(e.to_string()));
                    }
                    // Try to store last inserted rowid or id if available
                    // For SQLite, last_insert_rowid is u64
                    let id_val = values.get("id");
                    if let Some(id_val) = id_val {
                        match id_val {
                            Value::Int64(v) => inserted_ids.push(*v as u64),
                            Value::Int32(v) => inserted_ids.push(*v as u64),
                            Value::Int16(v) => inserted_ids.push(*v as u64),
                            Value::Int8(v) => inserted_ids.push(*v as u64),
                            Value::UInt64(v) => inserted_ids.push(*v),
                            Value::UInt32(v) => inserted_ids.push(*v as u64),
                            Value::UInt16(v) => inserted_ids.push(*v as u64),
                            Value::UInt8(v) => inserted_ids.push(*v as u64),
                            Value::String(_)
                            | Value::Float32(_)
                            | Value::Float64(_)
                            | Value::Bool(_)
                            | Value::Between(_, _)
                            | Value::Array(_)
                            | Value::Null => {}
                        }
                    } else {
                        #[cfg(feature = "sqlite")]
                        {
                            if let Ok(s) = result {
                                inserted_ids.push(s.last_insert_rowid() as u64);
                            }
                        }
                    }
                }
            }
        }

        #[cfg(any(feature = "mysql", feature = "sqlite"))]
        {
            if self.returning.is_empty() {
                return Ok(None);
            }

            // Fetch selected columns for all inserted ids
            let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
            let mut select_sql = get_dialect().returning_sql(select_sql, &self.returning);
            select_sql.push_str(format!(" FROM {} WHERE id = ?;", T::table_name()).as_str());

            for id in inserted_ids {
                #[cfg(feature = "mysql")]
                let q = sqlx::query(&select_sql).bind(id);

                #[cfg(feature = "sqlite")]
                let q = sqlx::query(&select_sql).bind(id as i64);

                let rows = q.fetch_all(&mut *conn).await;

                if let Err(e) = rows {
                    return Err(DatabaseError::QueryError(e.to_string()));
                }

                let rows = rows.unwrap();

                #[cfg(feature = "mysql")]
                let rows = Row::<T>::from_mysql_row(rows, None);

                #[cfg(feature = "sqlite")]
                let rows = Row::<T>::from_sqlite_row(rows, None);

                final_rows.extend(rows);
            }

            Ok(Some(final_rows))
        }

        #[cfg(feature = "postgres")]
        {
            // For PostgreSQL, if returning was used, rows are already in final_rows
            if !self.returning.is_empty() {
                Ok(Some(final_rows))
            } else if !inserted_ids.is_empty() {
                let select_sql = get_starting_sql(StartingSql::Select, T::table_name());
                let mut select_sql = get_dialect().returning_sql(select_sql, &self.returning);
                select_sql.push_str(format!(" FROM {} WHERE id = $1;", T::table_name()).as_str());

                for id in inserted_ids {
                    let q = sqlx::query(&select_sql).bind(id as i64);
                    let rows = q.fetch_all(&mut *conn).await;
                    if let Err(e) = rows {
                        return Err(DatabaseError::QueryError(e.to_string()));
                    }
                    let rows = rows.unwrap();

                    let rows = Row::<T>::from_postgres_row(rows, None);
                    final_rows.extend(rows);
                }

                Ok(Some(final_rows))
            } else {
                Ok(None)
                // Fetch selected columns for all inserted ids
            }
        }
    }
}
