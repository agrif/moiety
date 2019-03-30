use crate::future::*;

pub trait ResourceMap {
    type Handle;
    type Error;
    fn open_raw<'a, T: ResourceType + 'a>(&'a self, stack: Stack, typ: T, id: u16) -> FutureObjResult<'a, Self::Handle, Self::Error>;
}

pub trait ResourceType: Copy {
    type Data;
    fn name(&self) -> &'static str;
}

macro_rules! stack {
    { $($t:ident ( $name:expr, $letter:expr ), )* } => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum Stack {
            $($t),*
        }

        impl Stack {
            pub fn name(&self) -> &'static str {
                match self {
                    $( Stack::$t => $name, )*
                }
            }

            pub fn letter(&self) -> &'static str {
                match self {
                    $( Stack::$t => $letter, )*
                }
            }
        }
    };
}

stack!{
    A("aspit", "a"),
    B("bspit", "b"),
}


