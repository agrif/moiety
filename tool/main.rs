#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::*;

async fn go() -> Result<(), MhkError> {
    for_each_riven!(|r| => {
        println!("found resource type: {:?}", r);
    });
    let fs = LoggingFilesystem::new("root", LocalFilesystem::new("/home/agrif/vault/games/riven/"));
    let map = MhkMap::new(fs);
    let rs = Resources::new_with_map_error(map, MhkFormat);
    let resource = await!(rs.open(RivenStack::A, Riven::NAME, 2))?;
    println!("{:?}", Riven::NAME);
    println!("{:?}", resource);

    Ok(())
}

fn main() -> Result<(), MhkError> {
    futures::executor::block_on(go())
}
