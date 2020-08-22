use moiety::filesystem::{LocalFilesystem, LoggingFilesystem};
use moiety::{DirectMap, JsonFormat, CurFormat, PngFormat, MixedFormat, Resources};
use moiety::riven;

fn main() -> anyhow::Result<()> {
    smol::run(async {
        let fs = LocalFilesystem::new("/Users/agrif/vault/games/riven/");
        let outfs = LoggingFilesystem::new(
            "out",
            LocalFilesystem::new("./local/riven/"),
        );

        let map = riven::map_5cd(fs).await?;
        let outmap = DirectMap::new(outfs, MixedFormat {
            bitmap: PngFormat,
            cursor: CurFormat,
            record: JsonFormat(false),
        });

        let mut rs = Resources::new(map);
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
        rs.write_to(&mut outrs, riven::TCur).await?;
        Ok(())
    })
}
