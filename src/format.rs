use crate::future::*;
use crate::ResourceType;

pub trait Format<I> {
    type Error: failure::Fail;
}

pub trait FormatFor<I, R: ResourceType>: Format<I> {
    fn convert<'a>(&'a self, input: I) -> Fut<'a, Result<R::Data, Self::Error>> where I: 'a;

    fn extension<'a>(&'a self) -> Option<&'a str> {
        None
    }
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
    fn write<'a>(&'a self, input: I, fmt: &'a F) -> Fut<'a, Result<Vec<u8>, ConvertError<F::Error, Self::WriteError>>> where I: 'a, F: 'a;
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
    (Blst, Vec<ButtonMeta>, BLST, "BLST"),
    (Card, Card, CARD, "CARD"),
    (Name, Vec<Name>, NAME, "NAME"),
    (Plst, Vec<PictureMeta>, PLST, "PLST"),
});

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ButtonMeta {
    pub index: u16,
    pub enabled: u16,
    pub hotspot_id: u16,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Card {
    pub name_rec: i16,
    pub zip_mode_place: u16,
    pub script: std::collections::HashMap<Event, Vec<Command>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Event {
    MouseDown,
    MouseStillDown,
    MouseUp,
    MouseEnter,
    MouseWithin,
    MouseLeave,
    LoadCard,
    CloseCard,
    OpenCard,
    DisplayUpdate,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
pub enum Command {
    DrawBMP {
        tbmp_id: u16,
        left: u16,
        top: u16,
        right: u16,
        bottom: u16,
        u0: u16,
        u1: u16,
        u2: u16,
        u3: u16,
    },
    GotoCard {
        id: u16,
    },
    Conditional {
        var: u16,
        branches: std::collections::HashMap<u16, Vec<Command>>,
    },

    Dummy,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Name {
    pub unknown: u16,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PictureMeta {
    pub index: u16,
    pub bitmap_id: u16,
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

