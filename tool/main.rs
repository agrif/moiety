#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::*;
use riven::Resource;

async fn go() -> Result<(), MhkError> {
    let fs = filesystem::LocalFilesystem::new("/home/agrif/vault/games/riven/");
    let outfs = filesystem::LoggingFilesystem::new(
        "w",
        filesystem::LocalFilesystem::new("./local/"),
    );

    let map = MhkMap::new(
        fs,
        [
            (riven::Stack::A, vec!["a_Data.MHK", "a_Sounds.MHK"]),
            (riven::Stack::B, vec![
                "b_Data.MHK",
                "b2_data.MHK",
                "b_Sounds.MHK",
            ]),
            (riven::Stack::G, vec!["g_Data.MHK", "g_Sounds.MHK"]),
            (riven::Stack::J, vec![
                "j_Data1.MHK",
                "j_Data2.MHK",
                "j_Sounds.MHK",
            ]),
            (riven::Stack::O, vec!["o_Data.MHK", "o_Sounds.MHK"]),
            (riven::Stack::P, vec!["p_Data.MHK", "p_Sounds.MHK"]),
            (riven::Stack::R, vec!["r_Data.MHK", "r_Sounds.MHK"]),
            (riven::Stack::T, vec!["t_Data.MHK", "t_Sounds.MHK"]),
        ]
        .iter()
        .cloned()
        .collect(),
    );
    let outmap = DirectMap::new(outfs);

    let fmt = MhkFormat;
    //let outfmt = JsonFormat;
    let outfmt = YamlFormat;
    // let outfmt = MhkFormat;

    let rs = Resources::new_with_map_error(map, fmt);
    let mut outrs = Resources::new_with_format_error(outmap, outfmt);

    for_each_riven!(|r| => {
        let x = await!(rs.write_to(&mut outrs, r));
        println!("{:?}: {:?}", r, x);
        x.unwrap();
    });

    // let x = await!(rs.write_to(&mut outrs, riven::Resource::CARD));
    // println!("{:?}", x);

    Ok(())
}

fn main() -> Result<(), MhkError> { futures::executor::block_on(go()) }
