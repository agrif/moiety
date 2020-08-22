mod script;
pub use script::*;

mod blst;
pub use blst::*;

mod card;
pub use card::*;

mod flst;
pub use flst::*;

mod hspt;
pub use hspt::*;

mod mlst;
pub use mlst::*;

mod name;
pub use name::*;

mod plst;
pub use plst::*;

mod rmap;
pub use rmap::*;

mod sfxe;
pub use sfxe::*;

mod slst;
pub use slst::*;

mod tbmp;
pub use tbmp::*;

mod tcur;
pub use tcur::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Stack {
    A,
    B,
    G,
    J,
    O,
    P,
    R,
    T,
    Extras,
    Exe,
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
            Exe => "exe",
        }
    }

    fn all() -> Vec<Self> {
        use Stack::*;
        vec![A, B, G, J, O, P, R, T, Extras, Exe]
    }
}

pub async fn map_5cd<F>(mut fs: F) -> anyhow::Result<impl crate::ResourceMapList<Format=crate::mhk::MhkFormat, Stack=Stack>>
where
    F: crate::filesystem::Filesystem,
{
    let arcriven = smol::io::BlockOn::new(fs.open(&["arcriven.z"]).await?);
    let z = crate::filesystem::ZArchive::new(arcriven).await?;
    Ok(crate::mhk::MhkMap::new(
        (z, fs),
        [
            (Stack::A, vec!["a_Data.MHK", "a_Sounds.MHK"]),
            (Stack::B, vec!["b_Data.MHK", "b_Sounds.MHK", "b2_data.MHK"]),
            (Stack::G, vec!["g_Data.MHK", "g_Sounds.MHK"]),
            (Stack::J, vec!["j_Data1.MHK", "j_Data2.MHK", "j_Sounds.MHK"]),
            (Stack::O, vec!["o_Data.MHK", "o_Sounds.MHK"]),
            (Stack::P, vec!["p_Data.MHK", "p_Sounds.MHK"]),
            (Stack::R, vec!["r_Data.MHK", "r_Sounds.MHK"]),
            (Stack::T, vec!["t_Data.MHK", "t_Sounds.MHK"]),
            (Stack::Extras, vec!["Extras.MHK"]),
            (Stack::Exe, vec!["Riven.exe"]),
        ]
        .iter()
        .cloned()
        .collect(),
    ))
}
