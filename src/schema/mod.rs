mod column;

pub use column::Column;
pub use column::Value;

pub trait Schema {
    fn table_name() -> &'static str;
}

#[macro_export]
macro_rules! define_columns {
    ($(
        $struct_name:ident {
            $(
                $name:ident : $type:ty $( [ $($args:tt)* ] )?
            ),* $(,)?
        }
    )*) => {
        $(
            pub struct $struct_name;

            impl $struct_name {
                $(
                    pub fn $name() -> &'static Column<$type> {
                        static COL: std::sync::OnceLock<Column<$type>> = std::sync::OnceLock::new();
                        COL.get_or_init(|| {
                            Column::<$type>::new(stringify!($name))
                                $(.$($args)*)?
                        })
                    }
                )*
            }

            impl Schema for $struct_name {
                fn table_name() -> &'static str {
                    stringify!($struct_name)
                }
            }
        )*
    };
}
