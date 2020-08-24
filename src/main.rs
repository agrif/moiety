use moiety::filesystem::LocalFilesystem;
use moiety::riven;
use moiety::sdl;

use anyhow::Result;

fn main() -> Result<()> {
    smol::run(async {
        let fs = LocalFilesystem::new("/Users/agrif/vault/games/riven/");
        let map = riven::map_5cd(fs).await?;
        let game = riven::Riven::new(map).await?;

        sdl::Sdl::run(game).await?;

        Ok(())
    })
}
