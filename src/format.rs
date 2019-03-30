use crate::future::*;
use crate::ResourceType;

pub trait Format<I> {
    type Error;
}

pub trait FormatFor<I, R: ResourceType>: Format<I> {
    fn convert<'a>(&'a self, input: I) -> FutureObjResult<'a, R::Data, Self::Error> where I: 'a;
}

macro_rules! resources {
    ( $set_name:ident, { $(($enum:ident, $data:ty, $cname:ident, $name:expr),)* } ) => {
        pub enum $set_name<D> {
            $(
                $enum(refl::Id<D, $data>),
            )*
        }

        $(
            impl $set_name<$data> {
                pub const $cname: Self = $set_name::$enum(refl::Id::REFL);
            }
        )*

        impl<D> Copy for $set_name<D> {}

        impl<D> Clone for $set_name<D> {
            fn clone(&self) -> Self {
                match self {
                    $(
                        $set_name::$enum(ref id) => $set_name::$enum(id.clone()),
                    )*
                }
            }
        }

        impl<D> std::fmt::Debug for $set_name<D> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                        $set_name::$enum(_) => write!(f, stringify!($enum)),
                    )*
                }
            }
        }
        
        impl<D> ResourceType for $set_name<D> {
            type Data = D;
            fn name(&self) -> &'static str {
                match self {
                    $(
                        $set_name::$enum(_) => $name,
                    )*
                }
            }
        }
    }
}

resources!(Riven, {
    (Name, Vec<Name>, NAME, "NAME"),
});

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name {
    pub unknown: u16,
    pub name: String,
}
