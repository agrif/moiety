mod script;
pub use script::*;

mod blst;
pub use blst::*;

mod card;
pub use card::*;

mod hspt;
pub use hspt::*;

mod name;
pub use name::*;

mod plst;
pub use plst::*;

mod tbmp;
pub use tbmp::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Stack {
    A, B, G, J, O, P, R, T, Extras
}

impl crate::Stack for Stack {
    fn name(&self) -> &str {
        use Stack::*;
        match self {
            A => "aspit",
            B => "bspit",
            G => "gspit",
            J => "jspit",
            O => "ospit",
            P => "pspit",
            R => "rspit",
            T => "tspit",
            Extras => "extras",
        }
    }

    fn all() -> Vec<Self> {
        use Stack::*;
        vec![A, B, G, J, O, P, R, T] // FIXME extras, usually in arcriven.z
    }
}

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
    ].iter().cloned().collect()
}
