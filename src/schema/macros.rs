/// Helper macro: decides field type as `Option<T>` if `default_value(...)` or
/// `auto_increment()` is present in the column args; otherwise keeps it as `T`.
#[macro_export]
macro_rules! __lume_option_type {
    ($ty:ty) => { $ty };
    ($ty:ty, []) => { $ty };
    // If args contain default_value(...), make it Option
    ($ty:ty, [ default_value ( $($inner:tt)* ) $($tail:tt)* ]) => { Option<$ty> };
    // If args contain auto_increment(), make it Option
    ($ty:ty, [ auto_increment ( ) $($tail:tt)* ]) => { Option<$ty> };
    // Recurse through any other tokens
    ($ty:ty, [ $head:tt $($tail:tt)* ]) => { $crate::__lume_option_type!($ty, [ $($tail)* ]) };
}

/// Defines a database schema with type-safe columns and constraints.
///
/// This macro creates a schema struct that implements the [`Schema`] trait
/// and provides type-safe access to column definitions.
///
/// # Syntax
///
/// ```rust
/// use lume::define_schema;
/// use lume::schema::{Schema, ColumnInfo};
///
/// define_schema! {
///     TableName {
///         column_name: i32 [primary_key()],
///         another_column: String [not_null()],
///     }
/// }
/// ```
///
/// # Column Constraints
///
/// - `primary_key()` - Sets the column as primary key
/// - `not_null()` - Makes the column NOT NULL
/// - `unique()` - Adds a UNIQUE constraint
/// - `indexed()` - Creates an index on the column
/// - `default_value(value)` - Sets a default value
///
/// # Example
///
/// ```rust
/// use lume::define_schema;
/// use lume::schema::{Schema, ColumnInfo};
///
/// define_schema! {
///     User {
///         id: i32 [primary_key().not_null()],
///         username: String [not_null().unique()],
///         email: String [not_null()],
///         age: i32,
///         is_active: bool [default_value(true)],
///         created_at: i64 [not_null()],
///     }
/// }
///
/// // Access columns type-safely
/// let id_col = User::id();
/// let username_col = User::username();
///
/// // Get schema information
/// assert_eq!(User::table_name(), "User");
/// let columns = User::get_all_columns();
/// ```
///
/// # Generated Code
///
/// This macro generates:
/// - A struct with the given name
/// - Column accessor methods that return `&'static Column<T>`
/// - Implementation of the [`Schema`] trait
/// - Automatic table registration
#[macro_export]
macro_rules! define_schema {
    (
        $(
            $struct_name:ident {
            $(
                $name:ident: $type:ty $([ $($args:tt)* ])?
            ),* $(,)?
        }
    )*
    ) => {
    // Auto-register the table when the struct is defined
    #[allow(non_upper_case_globals)]
    static _REGISTER: std::sync::Once = std::sync::Once::new();
    use $crate::table::register_table;
    use $crate::schema::type_to_sql_string;
    use $crate::schema::DefaultToSql;
    use $crate::schema::Value;

        $(
        #[derive(Debug)]
        pub struct $struct_name {
            $(
                pub $name: $crate::__lume_option_type!($type $(, [ $($args)* ])? ),
            )*
        }

        paste::paste! {
            #[derive(Debug)]
            #[allow(unused)]
            pub struct [<Update $struct_name>] {
                $(
                    pub $name: Option<$type>,
                )*
            }

            impl Default for [<Update $struct_name>] {
                fn default() -> Self {
                    Self {
                        $(
                            $name: None,
                        )*
                    }
                }
            }

            impl [<Update $struct_name>] {
                $(
                    pub fn $name() -> &'static $crate::schema::Column<$type> {
                        static COL: std::sync::OnceLock<$crate::schema::Column<$type>> = std::sync::OnceLock::new();
                        COL.get_or_init(|| {
                            $crate::schema::Column::<$type>::new(stringify!($name), stringify!($struct_name))
                                $(.$($args)*)?
                        })
                    }
                )*

            }
            impl $crate::schema::UpdateTrait for [<Update $struct_name>] {

                fn get_updated(self) -> Vec<(&'static str, Value)> {
                    let mut vec = Vec::new();

                    $(
                        if self.$name.is_some() {
                            vec.push((stringify!($struct_name.$name), $crate::schema::convert_to_value(&self.$name)));
                        }
                    )*

                    vec
                }
            }



            impl $crate::schema::Schema for [<Update $struct_name>] {
                fn table_name() -> &'static str {
                    stringify!($struct_name)
                }

                fn values(&self) -> std::collections::HashMap<String, Value> {
                    let mut map = std::collections::HashMap::new();
                    $(
                        map.insert(
                            stringify!($name).to_string(),
                            $crate::schema::convert_to_value(&self.$name)
                        );
                    )*
                    map
                }
                fn ensure_registered() {
                    // Function-local static to avoid name collisions across macro expansions
                    static REGISTER: std::sync::Once = std::sync::Once::new();
                    REGISTER.call_once(|| {
                        register_table::<$struct_name>();
                    });
                }

                fn get_all_columns() -> Vec<$crate::schema::ColumnInfo<'static>> {
                    vec![
                        $(
                            {
                                let col = Self::$name();

                                $crate::schema::ColumnInfo {
                                    name: col.__internal_name(),
                                    data_type: type_to_sql_string::<$type>(),
                                    has_default: col.__internal_get_default().is_some(),
                                    default_sql: col.default_to_sql(),
                                    comment: col.__internal_get_comment(),
                                    charset: col.__internal_get_charset(),
                                    collate: col.__internal_get_collate(),
                                    validators: col.__internal_get_validators(),
                                    constraints: col.__internal_get_constraints(),
                                }
                            }
                        ),*
                    ]
                }
            }
        }

        paste::paste! {
            #[derive(Debug)]
            pub struct [<Select $struct_name>] {
                $(
                    pub $name: bool,
                )*
            }

            impl [<Select $struct_name>] {

                $(
                    fn $name(mut self) -> Self {
                        self.$name = true;
                        self
                    }
                )*

                #[allow(dead_code)]
                fn selected() -> Self {
                    Self {
                        $(
                            $name: false,
                        )*
                    }
                }

                #[allow(unused)]
                fn all(mut self) -> Self {
                    $(
                        self.$name = true;
                    )*

                    self
                }
            }

            impl Default for [<Select $struct_name>] {
                fn default() -> Self {
                    Self {
                        $(
                            $name: false,
                        )*
                    }
                }
            }

            impl $crate::schema::Select for [<Select $struct_name>] {
                fn default() -> Self {
                    Self {
                        $(
                            $name: true,
                        )*
                    }
                }


                fn get_selected(self) -> Vec<&'static str> {
                    let mut vec = Vec::new();

                    $(
                        if self.$name {
                            vec.push(stringify!($struct_name.$name))
                        }
                    )*

                    vec
                }
            }
        }


        impl $struct_name {
            $(
                pub fn $name() -> &'static $crate::schema::Column<$type> {
                    static COL: std::sync::OnceLock<$crate::schema::Column<$type>> = std::sync::OnceLock::new();
                    COL.get_or_init(|| {
                        $crate::schema::Column::<$type>::new(stringify!($name), stringify!($struct_name))
                            $(.$($args)*)?
                    })
                }
            )*
        }


        impl $crate::schema::Schema for $struct_name {
            fn table_name() -> &'static str {
                stringify!($struct_name)
            }

            fn values(&self) -> std::collections::HashMap<String, Value> {
                let mut map = std::collections::HashMap::new();
                $(
                    map.insert(
                        stringify!($name).to_string(),
                        $crate::schema::convert_to_value(&self.$name)
                    );
                )*
                map
            }
            fn ensure_registered() {
                // Function-local static to avoid name collisions across macro expansions
                static REGISTER: std::sync::Once = std::sync::Once::new();
                REGISTER.call_once(|| {
                    register_table::<$struct_name>();
                });
            }

            fn get_all_columns() -> Vec<$crate::schema::ColumnInfo<'static>> {
                vec![
                    $(
                        {
                            let col = Self::$name();

                            $crate::schema::ColumnInfo {
                                name: col.__internal_name(),
                                data_type: type_to_sql_string::<$type>(),
                                has_default: col.__internal_get_default().is_some(),
                                default_sql: col.default_to_sql(),
                                comment: col.__internal_get_comment(),
                                charset: col.__internal_get_charset(),
                                collate: col.__internal_get_collate(),
                                validators: col.__internal_get_validators(),
                                constraints: col.__internal_get_constraints(),
                            }
                        }
                    ),*
                ]
            }
        }
        )*
    };
}

#[macro_export]
/// Macro to generate SQL string conversions for enums used as custom SQL column types.
///
/// This macro implements the [`ToString`] and [`TryFrom<Value>`] traits for an enum,
/// enabling seamless conversion between enum variants and their string representations
/// in the database. It is intended for use with enums representing custom column types
/// that need to be stored as strings in SQL databases.
///
/// # Example
///
/// ```rust
/// use lume::enum_to_sql;
/// use lume::schema::Value;
///
/// #[derive(PartialEq)]
/// pub enum UserStatus {
///     Active,
///     Inactive,
///     Banned,
/// }
///
/// enum_to_sql!(UserStatus {
///     Active => "active",
///     Inactive => "inactive",
///     Banned => "banned",
/// });
///
/// assert_eq!(UserStatus::Active.to_string(), "active");
/// assert_eq!(
///     UserStatus::try_from(Value::String("banned".to_string())),
///     Ok(UserStatus::Banned)
/// );
/// ```
///
/// # Macro Usage
///
/// ```ignore
/// enum_to_sql!(EnumName {
///     Variant1 => "sql_value1",
///     Variant2 => "sql_value2",
///     // ...
/// });
/// ```
///
/// - Automatically implements:
///     - [`ToString`] for the enum mapping each variant to the specified string.
///     - [`TryFrom<Value>`] for the enum (converts a string value to the respective variant; returns `Err(())` if not matched).
///
/// [`ToString`]: std::string::ToString
/// [`TryFrom<Value>`]: std::convert::TryFrom
macro_rules! enum_to_sql {
    ($enum_name:ident { $($variant:ident => $str:expr),* $(,)? }) => {
        impl ToString for $enum_name {
            fn to_string(&self) -> String {
                match self {
                    $(
                        $enum_name::$variant => String::from($str),
                    )*
                }
            }
        }

        impl std::fmt::Debug for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl TryFrom<$crate::schema::Value> for $enum_name {
            type Error = ();

            fn try_from(value: $crate::schema::Value) -> Result<Self, Self::Error> {
                match value {
                    $crate::schema::Value::String(s) => match s.as_str() {
                        $(
                            $str => Ok($enum_name::$variant),
                        )*
                        _ => Err(()),
                    },
                    _ => Err(()),
                }
            }
        }

        impl $crate::schema::CustomSqlType for $enum_name {}
    };
}
