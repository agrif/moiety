use crate::future::*;

pub trait ResourceMap {
    type Handle;
    type Error;
    type Stack;
    fn open_raw<'a, T: ResourceType + 'a>(&'a self, stack: Self::Stack, typ: T, id: u16) -> FutureObjResult<'a, Self::Handle, Self::Error>;
}

pub trait ResourceType: Copy {
    type Data;
    fn name(&self) -> &'static str;
}

pub trait Stack: Copy + std::cmp::Eq + std::hash::Hash {
    fn name(&self) -> &'static str;
    fn letter(&self) -> &'static str;
}

macro_rules! stack {
    ( $tyname:ident, { $($t:ident ( $name:expr, $letter:expr ), )* } ) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum $tyname {
            $($t),*
        }

        impl Stack for $tyname {
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
        }
    };
}

stack!(RivenStack, {
    A("aspit", "a"),
    B("bspit", "b"),
});


