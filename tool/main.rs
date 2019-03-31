#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::*;

async fn go() -> Result<(), MhkError> {
    for_each_riven!(|r| => {
        println!("found resource type: {:?}", r);
    });
    
    let fs = LocalFilesystem::new("/home/agrif/vault/games/riven/");
    let outfs = LoggingFilesystem::new("w", LocalFilesystem::new("./local/"));
    
    let map = MhkMap::new(fs, [
        (RivenStack::A, vec!["a_Data.MHK", "a_Sounds.MHK"]),
        (RivenStack::B, vec!["b_Data.MHK", "b2_data.MHK", "b_Sounds.MHK"]),
        (RivenStack::G, vec!["g_Data.MHK", "g_Sounds.MHK"]),
        (RivenStack::J, vec!["j_Data1.MHK", "j_Data2.MHK", "j_Sounds.MHK"]),
        (RivenStack::O, vec!["o_Data.MHK", "o_Sounds.MHK"]),
        (RivenStack::P, vec!["p_Data.MHK", "p_Sounds.MHK"]),
        (RivenStack::R, vec!["r_Data.MHK", "r_Sounds.MHK"]),
        (RivenStack::T, vec!["t_Data.MHK", "t_Sounds.MHK"]),
    ].iter().cloned().collect());
    let outmap = DirectMap::new(outfs);

    let fmt = MhkFormat;
    let outfmt = JsonFormat;
    
    let rs = Resources::new_with_map_error(map, fmt);
    let mut outrs = Resources::new_with_format_error(outmap, outfmt);
    
    let resource = await!(rs.open(RivenStack::A, Riven::NAME, 2))?;
    println!("{:?}", Riven::NAME);
    println!("{:?}", resource);

    let x = await!(rs.write_to(&mut outrs, Riven::NAME));

    println!("{:?}", x);

    Ok(())
}

fn main() -> Result<(), MhkError> {
    futures::executor::block_on(go())
}
