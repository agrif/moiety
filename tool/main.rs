#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::*;
use riven::Resource;

async fn go() -> Result<(), MhkError> {
    let fs = filesystem::LocalFilesystem::new("/home/agrif/vault/games/riven/");
    let outfs = filesystem::LoggingFilesystem::new(
        "w",
        filesystem::LocalFilesystem::new("./web/local/"),
    );

    let map = MhkMap::new(
        fs,
        riven::map_5cd(),
    );
    let outmap = DirectMap::new(outfs);

    let fmt = MhkFormat;
    let outfmt = riven::Format {
        blst: JsonFormat,
        card: JsonFormat,
        name: JsonFormat,
        plst: JsonFormat,
        tbmp: PngFormat,
    };

    let rs = Resources::new(map, fmt);
    let mut outrs = Resources::new(outmap, outfmt);

    for_each_riven!(|r| => {
       let x = await!(rs.write_to(&mut outrs, r));
       println!("{:?}: {:?}", r, x);
       x.unwrap();
    });

    // let x = await!(rs.write_resource_to(
    //    &mut outrs,
    //    riven::Stack::B,
    //    riven::Resource::TBMP,
    //    44
    //));
    // x.unwrap();

    // let x = await!(rs.write_stack_to(&mut outrs, riven::Stack::B2, riven::Resource::TBMP));
    // x.unwrap();

    // let x = await!(rs.write_to(&mut outrs, riven::Resource::TBMP));
    // println!("{:?}", x);

    Ok(())
}

fn main() -> Result<(), MhkError> { futures::executor::block_on(go()) }
