use time::macros::format_description;

use crate::schema::{Column, CustomSqlType};

/// Trait for converting column default values to SQL representation.
///
/// This trait is implemented for all supported column types to provide
/// proper SQL formatting of default values in CREATE TABLE statements.
pub trait DefaultToSql {
    /// Converts the column's default value to its SQL representation.
    ///
    /// Returns `None` if the column has no default value.
    ///
    /// # Returns
    ///
    /// - `Some(String)`: The SQL representation of the default value
    /// - `None`: If no default value is set
    fn default_to_sql(&self) -> Option<DefaultValueEnum<String>>;
}

// Macro to implement DefaultToSql for numeric types and their Vec variants
macro_rules! impl_default_to_sql_numeric {
    ($($t:ty),*) => {
        $(
            impl DefaultToSql for $crate::schema::Column<$t> {
                fn default_to_sql(&self) -> Option<DefaultValueEnum<String>> {
                    self.__internal_get_default().map(|v| match v {
                            DefaultValueEnum::Value(val) => DefaultValueEnum::Value(val.to_string()),
                            DefaultValueEnum::CurrentTimestamp => DefaultValueEnum::CurrentTimestamp,
                            DefaultValueEnum::Random => DefaultValueEnum::Random,
                    })
                }
            }

            #[cfg(feature = "postgres")]
            impl DefaultToSql for Column<Vec<$t>> {
                fn default_to_sql(&self) -> Option<DefaultValueEnum<String>> {
                    self.get_default().map(|v| match v {
                        DefaultValueEnum::Value(vec) => {
                            let items = vec.iter()
                                 .map(|item| item.to_string())
                                 .collect::<Vec<_>>();
                            DefaultValueEnum::Value(format!("ARRAY[{}]", items.join(", ")))
                        }
                        DefaultValueEnum::CurrentTimestamp => DefaultValueEnum::CurrentTimestamp,
                        DefaultValueEnum::Random => DefaultValueEnum::Random,
                    })
                }
            }

        )*
    };
}

// Implement for all numeric types
impl_default_to_sql_numeric!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);

// Implement for String (needs special escaping)
impl DefaultToSql for Column<String> {
    fn default_to_sql(&self) -> Option<DefaultValueEnum<String>> {
        self.__internal_get_default().map(|v| {
            if let DefaultValueEnum::Value(v) = v {
                DefaultValueEnum::Value(format!("'{}'", v.replace('\'', "''")))
            } else {
                DefaultValueEnum::Value("".to_string())
            }
        })
    }
}

impl DefaultToSql for Column<time::OffsetDateTime> {
    fn default_to_sql(&self) -> Option<DefaultValueEnum<String>> {
        let datetime = self.__internal_get_default();

        match datetime {
            None => None,
            Some(datetime) => {
                let format = format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"
                );
                if let DefaultValueEnum::Value(datetime) = datetime {
                    let mysql_datetime = datetime.format(&format).unwrap();
                    Some(DefaultValueEnum::Value(format!("'{}'", mysql_datetime)))
                } else {
                    None
                }
            }
        }
    }
}

// Implement for Vec<String> (needs special escaping)
#[cfg(feature = "postgres")]
impl DefaultToSql for Column<Vec<String>> {
    fn default_to_sql(&self) -> Option<DefaultValueEnum<std::string::String>> {
        match self.get_default() {
            Some(DefaultValueEnum::Value(vec)) => {
                let escaped = vec
                    .iter()
                    .map(|s| format!("'{}'", s.replace('\'', "''")))
                    .collect::<Vec<_>>();
                Some(DefaultValueEnum::Value(format!(
                    "ARRAY[{}]",
                    escaped.join(", ")
                )))
            }
            Some(DefaultValueEnum::CurrentTimestamp) => Some(DefaultValueEnum::CurrentTimestamp),
            Some(DefaultValueEnum::Random) => Some(DefaultValueEnum::Random),
            None => None,
        }
    }
}

// Implement for bool (needs TRUE/FALSE)
impl DefaultToSql for Column<bool> {
    fn default_to_sql(&self) -> Option<DefaultValueEnum<String>> {
        self.__internal_get_default().map(|v| {
            if let DefaultValueEnum::Value(v) = v {
                if *v {
                    DefaultValueEnum::Value("TRUE".to_string())
                } else {
                    DefaultValueEnum::Value("FALSE".to_string())
                }
            } else {
                DefaultValueEnum::Value("FALSE".to_string())
            }
        })
    }
}

// Implement for Vec<bool>
#[cfg(feature = "postgres")]
impl DefaultToSql for Column<Vec<bool>> {
    fn default_to_sql(&self) -> Option<DefaultValueEnum<std::string::String>> {
        match self.get_default() {
            Some(DefaultValueEnum::Value(vec)) => {
                let items = vec
                    .iter()
                    .map(|b| if *b { "TRUE" } else { "FALSE" })
                    .collect::<Vec<_>>();
                Some(DefaultValueEnum::Value(format!(
                    "ARRAY[{}]",
                    items.join(", ")
                )))
            }
            Some(DefaultValueEnum::CurrentTimestamp) => Some(DefaultValueEnum::CurrentTimestamp),
            Some(DefaultValueEnum::Random) => Some(DefaultValueEnum::Random),
            None => None,
        }
    }
}

// Generic implementation for user-defined types (enums, etc.)
// Users must implement CustomSqlType for their types to use this
impl<T> DefaultToSql for Column<T>
where
    T: ToString + CustomSqlType,
{
    fn default_to_sql(&self) -> Option<DefaultValueEnum<String>> {
        self.__internal_get_default().map(|v| {
            if let DefaultValueEnum::Value(v) = v {
                DefaultValueEnum::Value(v.to_string())
            } else {
                DefaultValueEnum::Value("".to_string())
            }
        })
    }
}

// Generic implementation for Vec<T> where T is a user-defined type
#[cfg(feature = "postgres")]
impl<T> DefaultToSql for Column<Vec<T>>
where
    T: ToString + CustomSqlType,
{
    fn default_to_sql(&self) -> Option<DefaultValueEnum<std::string::String>> {
        self.get_default().map(|v| match v {
            DefaultValueEnum::Value(vec) => {
                let items = vec.iter().map(|item| item.to_string()).collect::<Vec<_>>();
                DefaultValueEnum::Value(format!("ARRAY[{}]", items.join(", ")))
            }
            DefaultValueEnum::CurrentTimestamp => DefaultValueEnum::CurrentTimestamp,
            DefaultValueEnum::Random => DefaultValueEnum::Random,
        })
    }
}

/// Represents different types of default values that can be set on a column.
///
/// This enum allows columns to have literal default values, database-generated
/// timestamps, or random values like UUIDs, depending on the column type and backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultValueEnum<T> {
    /// Use the database's current timestamp (e.g., `CURRENT_TIMESTAMP` in SQL).
    CurrentTimestamp,
    /// Use a database-generated random value (e.g., `UUID()` in MySQL, `gen_random_uuid()` in PostgreSQL).
    Random,
    /// Use a specific literal value provided by the user.
    Value(T),
}
