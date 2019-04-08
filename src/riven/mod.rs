use crate::future::*;

mod script;
pub use script::*;

mod blst;
pub use blst::*;
mod card;
pub use card::*;
mod name;
pub use name::*;
mod plst;
pub use plst::*;
mod tbmp;
pub use tbmp::*;

resources!(Resource, Format, FormatError, for_each_riven, {
    (Blst, Vec<ButtonMeta>, BLST, TBlst, blst, BlstError, "BLST"),
    (Card, Card, CARD, TCard, card, CardError, "CARD"),
    (Name, Vec<Name>, NAME, TName, name, NameError, "NAME"),
    (Plst, Vec<PictureMeta>, PLST, TPlst, plst, PlstError, "PLST"),
    (TBmp, Bitmap, TBMP, TTBmp, tbmp, TBmpError, "tBMP"),
});

stack!(Stack, {
    A("aspit", "a"),
    B("bspit", "b"),
    B2("b2spit", "b2"),
    G("gspit", "g"),
    J("jspit", "j"),
    O("ospit", "o"),
    P("pspit", "p"),
    R("rspit", "r"),
    T("tspit", "t"),
});
