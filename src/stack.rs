pub trait Stack: Copy + std::cmp::Eq + std::hash::Hash {
    fn name(&self) -> &'static str;
    fn letter(&self) -> &'static str;
    fn all() -> Vec<Self>;
}

#[macro_export]
macro_rules! stack {
    ( $tyname:ident, { $($t:ident ( $name:expr, $letter:expr ), )* } ) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum $tyname {
            $($t),*
        }

        impl $crate::Stack for $tyname {
            fn name(&self) -> &'static str {
                match self {
                    $( $tyname::$t => $name, )*
                }
            }

            fn letter(&self) -> &'static str {
                match self {
                    $( $tyname::$t => $letter, )*
                }
            }

            fn all() -> Vec<Self> {
                vec![$( $tyname::$t ),*]
            }
        }
    };
}
