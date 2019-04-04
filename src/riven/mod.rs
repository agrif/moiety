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

resources!(Resource, for_each_riven, {
    (Blst, Vec<ButtonMeta>, BLST, "BLST"),
    (Card, Card, CARD, "CARD"),
    (Name, Vec<Name>, NAME, "NAME"),
    (Plst, Vec<PictureMeta>, PLST, "PLST"),
});

stack!(Stack, {
    A("aspit", "a"),
    B("bspit", "b"),
    G("gspit", "g"),
    J("jspit", "j"),
    O("ospit", "o"),
    P("pspit", "p"),
    R("rspit", "r"),
    T("tspit", "t"),
});
