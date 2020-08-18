use moiety::filesystem::{LocalFilesystem, LoggingFilesystem};
use moiety::{MhkMap, DirectMap, JsonFormat, PngFormat, MixedFormat, Resources};
use moiety::riven;

fn main() -> anyhow::Result<()> {
    smol::run(async {
        let fs = LocalFilesystem::new("/Users/agrif/vault/games/riven/");
        let outfs = LoggingFilesystem::new(
            "out",
            LocalFilesystem::new("./local/riven/"),
        );

        let map = MhkMap::new(fs, riven::map_5cd());
        let outmap = DirectMap::new(outfs, MixedFormat {
            bitmap: PngFormat,
            record: JsonFormat(false),
        });

        let rs = Resources::new(map);
        let mut outrs = Resources::new(outmap);

        rs.write_to(&mut outrs, riven::TBlst).await?;
        rs.write_to(&mut outrs, riven::TCard).await?;
        rs.write_to(&mut outrs, riven::TFlst).await?;
        rs.write_to(&mut outrs, riven::THspt).await?;
        rs.write_to(&mut outrs, riven::TMlst).await?;
        rs.write_to(&mut outrs, riven::TName).await?;
        rs.write_to(&mut outrs, riven::TPlst).await?;
        rs.write_to(&mut outrs, riven::TRmap).await?;
        rs.write_to(&mut outrs, riven::TSfxe).await?;
        rs.write_to(&mut outrs, riven::TSlst).await?;
        rs.write_to(&mut outrs, riven::TBmp).await?;
        Ok(())
    })
}
