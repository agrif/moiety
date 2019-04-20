#![feature(async_await)]
#![feature(await_macro)]
#![feature(futures_api)]

use moiety::{
    display::Display,
    *,
};

async fn go() -> Result<(), sdl::SdlError> {
    let fs = filesystem::LocalFilesystem::new("/home/agrif/vault/games/riven/");
    let map = MhkMap::new(fs, riven::map_5cd());
    let fmt = MhkFormat;
    let rs = Resources::new(map, fmt);

    let mut display = moiety::sdl::Display::new("Moiety", 608, 392)?;
    let raw_bmp =
        await!(rs.open(riven::Stack::B, riven::Resource::TBMP, 3)).unwrap();
    let bmp = display.transfer(&raw_bmp)?;

    display.draw(&bmp, 0, 0, 608, 392);
    display.flip();

    while display.events()? {}

    Ok(())
}

fn main() -> Result<(), sdl::SdlError> { futures::executor::block_on(go()) }
