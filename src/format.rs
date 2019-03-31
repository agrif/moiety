use crate::future::*;
use crate::ResourceType;

pub trait Format<I> {
    type Error: failure::Fail;
}

pub trait FormatFor<I, R: ResourceType>: Format<I> {
    fn convert<'a>(&'a self, input: I) -> FutureObjResult<'a, R::Data, Self::Error> where I: 'a;
}

#[derive(Fail, Debug)]
pub enum ConvertError<R: failure::Fail, W: failure::Fail> {
    #[fail(display = "Error reading: {}", _0)]
    Read(#[cause] R),
    #[fail(display = "Error writing: {}", _0)]
    Write(#[cause] W),
}

pub trait FormatWriteFor<I, R: ResourceType, F: FormatFor<I, R>> {
    type WriteError: failure::Fail;
    fn write<'a>(&'a self, input: I, fmt: &'a F) -> FutureObjResult<'a, Vec<u8>, ConvertError<F::Error, Self::WriteError>> where I: 'a, F: 'a;
}

#[macro_export]
macro_rules! resources {
    ( $set_name:ident, $macro_name:ident, { $(($enum:ident, $data:ty, $cname:ident, $name:expr),)* } ) => {
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

        #[macro_export]
        macro_rules! $macro_name {
            ( |$n:ident| => $body:block ) => {
                $(
                    {
                        let $n = $set_name::$enum(refl::Id::REFL);
                        $body;
                    }
                )*
            }
        }

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
                        $set_name::$enum(_) => {
                            write!(f, stringify!($set_name))?;
                            write!(f, "::")?;
                            write!(f, stringify!($cname))
                        },
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

resources!(Riven, for_each_riven, {
    (Name, Vec<Name>, NAME, "NAME"),
});

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Name {
    pub unknown: u16,
    pub name: String,
}