#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::*;

async fn go() -> Result<(), MhkError> {
    for_each_riven!(|r| => {
        println!("found resource type: {:?}", r);
    });
    
    let fs = LoggingFilesystem::new("r", LocalFilesystem::new("/home/agrif/vault/games/riven/"));
    let outfs = LoggingFilesystem::new("w", LocalFilesystem::new("./local/"));
    
    let map = MhkMap::new(fs);
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
