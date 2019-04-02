use crate::FormatFor;
use crate::future::*;

pub trait ResourceMap {
    type Handle;
    type Error: failure::Fail;
    type Stack;
    fn open_raw<'a, T: ResourceType + 'a, F: FormatFor<Self::Handle, T>>(&'a self, fmt: &'a F, stack: Self::Stack, typ: T, id: u16) -> Fut<'a, Result<Self::Handle, Self::Error>>;
}

pub trait ResourceMapList: ResourceMap {
    fn list<'a, T: ResourceType + 'a>(&'a self, stack: Self::Stack, typ: T) -> Fut<'a, Result<Vec<u16>, Self::Error>>;
}

pub trait ResourceMapWrite: ResourceMap {
    fn write_raw<'a, T: ResourceType + 'a, F: FormatFor<Self::Handle, T>>(&'a mut self, fmt: &'a F, stack: Self::Stack, typ: T, id: u16, data: &'a [u8]) -> Fut<'a, Result<(), Self::Error>>;
}

pub trait ResourceType: Copy {
    type Data;
    fn name(&self) -> &'static str;
}

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

            fn all() -> Vec<Self> {
                vec![$( $tyname::$t ),*]
            }
        }
    };
}

stack!(RivenStack, {
    A("aspit", "a"),
    B("bspit", "b"),
    G("gspit", "g"),
    J("jspit", "j"),
    O("ospit", "o"),
    P("pspit", "p"),
    R("rspit", "r"),
    T("tspit", "t"),
});


