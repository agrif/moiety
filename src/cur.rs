use crate::{Cursor, Format, FormatWrite};

use anyhow::Result;
use smol::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CurFormat;

#[async_trait::async_trait(?Send)]
impl<R, I> Format<R, I, Cursor> for CurFormat
where
    I: AsyncRead + Unpin,
{
    fn extension(&self, _res: &R) -> Option<&str> {
        Some(".cur")
    }
    async fn parse(&self, _res: &R, input: &mut I) -> Result<Cursor> {
        let mut buf = Vec::with_capacity(1 << 10);
        input.read_to_end(&mut buf).await?;
        let inp = std::io::Cursor::new(buf);

        let icon_dir = ico::IconDir::read(inp)?;
        let icons = icon_dir.entries();
        if icons.len() == 0 {
            anyhow::bail!("empty cur file");
        }
        let icon = icons[0].decode()?;
        if let Some(hotspot) = icon.cursor_hotspot() {
            let data = palette::Pixel::from_raw_slice(icon.rgba_data())
                .iter().cloned().collect();
            Ok(Cursor {
                width: icon.width() as u16,
                height: icon.height() as u16,
                hotspot,
                data: data,
            })
        } else {
            anyhow::bail!("cursor does not have hotspot");
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<Fi, R, I> FormatWrite<Fi, R, I, Cursor> for CurFormat
where
    Fi: Format<R, I, Cursor>,
    I: AsyncRead + Unpin,
{
    async fn convert(&self, fmti: &Fi, res: &R, input: &mut I)
                     -> Result<Vec<u8>>
    {
        let cur = fmti.parse(res, input).await?;
        let mut out = std::io::Cursor::new(Vec::new());

        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Cursor);
        let mut icon = ico::IconImage::from_rgba_data(
            cur.width as u32,
            cur.height as u32,
            palette::Pixel::into_raw_slice(&cur.data)
                .iter().cloned().collect(),
        );
        icon.set_cursor_hotspot(Some(cur.hotspot));
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon)?);
        icon_dir.write(&mut out)?;
        Ok(out.into_inner())
    }
}
