mod column;

use std::marker::PhantomData;

use crate::table::TableDefinition;
pub use column::Column;
pub use column::Value;

pub trait Schema {
    fn table_name() -> &'static str;
    fn get_all_columns() -> Vec<ColumnInfo>;
    fn ensure_registered();
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: &'static str,
    pub data_type: &'static str,
    pub nullable: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub indexed: bool,
    pub has_default: bool,
    pub default_sql: Option<String>,
}

#[macro_export]
macro_rules! define_schema {
    (
        $struct_name:ident {
            $(
                $name:ident: $type:ty $([ $($args:tt)* ])?
            ),* $(,)?
        }
    ) => {
        #[derive(Debug)]
        pub struct $struct_name;

        impl $struct_name {
            $(
                pub fn $name() -> &'static Column<$type> {
                    static COL: std::sync::OnceLock<Column<$type>> = std::sync::OnceLock::new();
                    COL.get_or_init(|| {
                        Column::<$type>::new(stringify!($name), stringify!($struct_name))
                            $(.$($args)*)?
                    })
                }
            )*
        }

        impl Schema for $struct_name {
            fn table_name() -> &'static str {
                stringify!($struct_name)
            }

            fn ensure_registered() {
                _REGISTER.call_once(|| {
                    register_table::<$struct_name>();
                });
            }

            fn get_all_columns() -> Vec<ColumnInfo> {
                vec![
                    $(
                        {
                            let col = Self::$name();
                            ColumnInfo {
                                name: col.name(),
                                data_type: define_schema!(@rust_type_to_sql $type),
                                nullable: col.is_nullable(),
                                unique: col.is_unique(),
                                primary_key: col.is_primary_key(),
                                indexed: col.is_indexed(),
                                has_default: col.get_default().is_some(),
                                default_sql: match col.get_default() {
                                    Some(default) => define_schema!(@default_to_sql $type, Some(default)),
                                    None => define_schema!(@default_to_sql $type, None),
                                },
                            }
                        }
                    ),*
                ]
            }
        }

        // Auto-register the table when the struct is defined
        #[allow(non_upper_case_globals)]
        static _REGISTER: std::sync::Once = std::sync::Once::new();
        use lume::table::register_table;

        impl $struct_name {

        }
    };

    // Convert Rust types to SQL types
    (@rust_type_to_sql String) => { "TEXT" };
    (@rust_type_to_sql i32) => { "INTEGER" };
    (@rust_type_to_sql i64) => { "BIGINT" };
    (@rust_type_to_sql f32) => { "REAL" };
    (@rust_type_to_sql f64) => { "DOUBLE PRECISION" };
    (@rust_type_to_sql bool) => { "BOOLEAN" };
    (@rust_type_to_sql $other:ty) => { "TEXT" }; // fallback

    // Convert default values to SQL
    (@default_to_sql String, Some($default:expr)) => {
        Some(format!("'{}'", $default.replace('\'', "''")))
    };
    (@default_to_sql i32, Some($default:expr)) => {
        Some($default.to_string())
    };
    (@default_to_sql i64, Some($default:expr)) => {
        Some($default.to_string())
    };
    (@default_to_sql f32, Some($default:expr)) => {
        Some($default.to_string())
    };
    (@default_to_sql f64, Some($default:expr)) => {
        Some($default.to_string())
    };
    (@default_to_sql bool, Some($default:expr)) => {
        Some($default.to_string())
    };
    (@default_to_sql $type:ty, None) => { None };
    (@default_to_sql $type:ty, Some($default:expr)) => {
        Some("NULL".to_string()) // fallback
    };
}

pub(crate) struct SchemaWrapper<T: Schema> {
    _phantom: PhantomData<T>,
}

impl<T: Schema> SchemaWrapper<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Schema + Sync + Send> TableDefinition for SchemaWrapper<T> {
    fn table_name(&self) -> &'static str {
        T::table_name()
    }

    fn get_columns(&self) -> Vec<ColumnInfo> {
        T::get_all_columns()
    }

    fn to_create_sql(&self) -> String {
        let table_name = self.table_name();
        let columns = self.get_columns();

        let mut sql = format!("CREATE TABLE {} (\n", table_name);

        let column_definitions: Vec<String> = columns
            .iter()
            .map(|col| {
                let mut def = format!("    {} {}", col.name, col.data_type);

                if col.primary_key {
                    def.push_str(" PRIMARY KEY");
                }

                if !col.nullable && !col.primary_key {
                    def.push_str(" NOT NULL");
                }

                if col.unique && !col.primary_key {
                    def.push_str(" UNIQUE");
                }

                if let Some(ref default) = col.default_sql {
                    def.push_str(&format!(" DEFAULT {}", default));
                }

                def
            })
            .collect();

        sql.push_str(&column_definitions.join(",\n"));
        sql.push_str("\n);");

        // Add indexes
        let indexes: Vec<String> = columns
            .iter()
            .filter(|col| col.indexed && !col.primary_key)
            .map(|col| {
                format!(
                    "CREATE INDEX idx_{}_{} ON {} ({});",
                    table_name, col.name, table_name, col.name
                )
            })
            .collect();

        if !indexes.is_empty() {
            sql.push_str("\n\n");
            sql.push_str(&indexes.join("\n"));
        }

        sql
    }
}

pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn TableDefinition>;
}

impl<T: ?Sized + TableDefinition + Clone + 'static> CloneBox for T {
    fn clone_box(&self) -> Box<dyn TableDefinition> {
        Box::new(self.clone())
    }
}
