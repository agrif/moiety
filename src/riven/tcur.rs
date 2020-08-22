use crate::{Cursor, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_le_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::{AsyncRead, AsyncReadExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TCur;

impl ResourceType for TCur {
    type Data = Cursor;
    fn name(&self) -> &str {
        "tCUR"
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CurHeader {
    hotspot: (u16, u16),
    headersize: u32,
    width: u32,
    height: u32,
    planes: u16,
    bpp: u16,
    compression: u32,
    size: u32,
    xres: u32,
    yres: u32,
    colors: u32,
    u0: u32,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TCur, I, Cursor> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TCur, input: &mut I) -> Result<Cursor> {
        let mut header: CurHeader = deserialize_le_from(input).await?;
        if header.headersize != 40 {
            anyhow::bail!("bad cursor header");
        }
        if header.planes > 1 {
            anyhow::bail!("bad cursor header");
        }
        if header.compression != 0 {
            anyhow::bail!("can't decompress cursor");
        }

        header.height = header.height / 2;
        if header.colors == 0 {
            header.colors = 1 << header.bpp;
        }

        // read palette
        let mut palette = Vec::with_capacity(header.colors as usize);
        for _ in 0..header.colors {
            let mut color = [0; 4];
            input.read_exact(&mut color).await?;
            palette.push(palette::Srgb::new(color[2], color[1], color[0]));
        }

        // read XOR map
        let mut xor = vec![0; (header.width * header.height) as usize];
        match header.bpp {
            1 => {
                if header.width % 8 != 0 {
                    anyhow::bail!("packed cursor pixels not aligned");
                }
                let packed = xor.len() / 8;
                let start = xor.len() - packed;
                input.read_exact(&mut xor[start..]).await?;
                for i in 0..xor.len() {
                    if xor[start + (i / 8)] & (1 << (7 - i % 8)) > 0 {
                        xor[i] = 1;
                    } else {
                        xor[i] = 0;
                    }
                }
            }
            8 => input.read_exact(&mut xor).await?,
            _ => anyhow::bail!("cursor bpp {:?} unsupported", header.bpp),
        }

        // read AND map
        let width_packed = (header.width + 7) / 8;
        let mut and = vec![0; (width_packed * header.height) as usize];
        input.read_exact(&mut and).await?;

        // combine XOR and palette and AND into an image
        let mut image = Vec::with_capacity((header.width * header.height) as usize);
        for y in 0..header.height {
            for x in 0..header.width {
                let i = ((header.height - y - 1) * header.width + x) as usize;
                if xor[i] as usize >= palette.len() {
                    anyhow::bail!("bad cursor data");
                }
                let mut alpha: u8 = 255;
                if and[i / 8] & (1 << (7 - x % 8)) > 0 {
                    alpha = 0;
                }
                image.push(palette::Alpha {
                    color: palette[xor[i] as usize],
                    alpha: alpha,
                });
            }
        }

        Ok(Cursor {
            width: header.width as u16,
            height: header.height as u16,
            hotspot: header.hotspot,
            data: image,
        })
    }
}
