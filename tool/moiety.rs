#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::{
    display::Display,
    *,
};

async fn go() -> Result<(), riven::RivenError<MhkError, MhkError, sdl::SdlError>>
{
    let fs = filesystem::LocalFilesystem::new("/home/agrif/vault/games/riven/");
    let map = MhkMap::new(fs, riven::map_5cd());
    let fmt = MhkFormat;
    let rs = Resources::new(map, fmt);

    let game = riven::Riven::new(rs);
    // FIXME error handling
    let mut runner = sdl::SdlRunner::new("Moiety", 608, 392).unwrap();
    await!(runner.run(game))?;
    Ok(())
}

fn main() -> Result<(), impl failure::Fail> {
    futures::executor::block_on(go())
}