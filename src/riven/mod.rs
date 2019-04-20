use crate::{
    display::Bitmap,
    future::*,
};

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

mod game;
pub use game::*;

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
    G("gspit", "g"),
    J("jspit", "j"),
    O("ospit", "o"),
    P("pspit", "p"),
    R("rspit", "r"),
    T("tspit", "t"),
});

pub fn map_5cd() -> std::collections::HashMap<Stack, Vec<&'static str>> {
    [
        (Stack::A, vec!["a_Data.MHK", "a_Sounds.MHK"]),
        (Stack::B, vec!["b_Data.MHK", "b_Sounds.MHK", "b2_data.MHK"]),
        (Stack::G, vec!["g_Data.MHK", "g_Sounds.MHK"]),
        (Stack::J, vec!["j_Data1.MHK", "j_Data2.MHK", "j_Sounds.MHK"]),
        (Stack::O, vec!["o_Data.MHK", "o_Sounds.MHK"]),
        (Stack::P, vec!["p_Data.MHK", "p_Sounds.MHK"]),
        (Stack::R, vec!["r_Data.MHK", "r_Sounds.MHK"]),
        (Stack::T, vec!["t_Data.MHK", "t_Sounds.MHK"]),
    ]
    .iter()
    .cloned()
    .collect()
}
