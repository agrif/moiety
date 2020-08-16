use crate::{Bitmap, Format, FormatWrite};

use anyhow::Result;
use smol::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PngFormat;

#[async_trait::async_trait(?Send)]
impl<R, I> Format<R, I, Bitmap> for PngFormat
where
    I: AsyncRead + Unpin,
{
    fn extension(&self, _res: &R) -> Option<&str> {
        Some(".png")
    }
    async fn parse(&self, _res: &R, input: &mut I) -> Result<Bitmap> {
        let mut buf = Vec::with_capacity(1 << 16);
        input.read_to_end(&mut buf).await?;
        let mut dec = png::Decoder::new(std::io::Cursor::new(buf));
        dec.set_transformations(
            png::Transformations::EXPAND
                | png::Transformations::STRIP_ALPHA,
        );
        let (info, mut reader) = dec.read_info()?;
        let framesize = (info.width * info.height * 3) as usize;
        let mut data = Vec::with_capacity(framesize);
        unsafe {
            data.set_len(framesize);
            reader.next_frame(&mut data)?;
        }

        Ok(Bitmap {
            width: info.width as u16,
            height: info.height as u16,
            palette: None,
            data: palette::Pixel::from_raw_slice(&data).to_owned(),
        })
    }
}

#[async_trait::async_trait(?Send)]
impl<Fi, R, I> FormatWrite<Fi, R, I, Bitmap> for PngFormat
where
    Fi: Format<R, I, Bitmap>,
    I: AsyncRead + Unpin,
{
    async fn convert(&self, fmti: &Fi, res: &R, input: &mut I)
                     -> Result<Vec<u8>>
    {
        let bmp = fmti.parse(res, input).await?;
        let mut buf = std::io::Cursor::new(Vec::with_capacity(
            bmp.width as usize * bmp.height as usize * 3,
        ));
        {
            let mut enc = png::Encoder::new(
                &mut buf,
                bmp.width as u32,
                bmp.height as u32,
            );
            enc.set_depth(png::BitDepth::Eight);
            if let Some(pal) = bmp.palette {
                enc.set_color(png::ColorType::Indexed);
                enc.set_palette(palette::Pixel::into_raw_slice(&pal.palette)
                                .to_owned());
                let mut writer = enc.write_header()?;
                writer.write_image_data(&pal.image[..])?;
            } else {
                enc.set_color(png::ColorType::RGB);
                let mut writer = enc.write_header()?;
                writer.write_image_data(palette::Pixel::into_raw_slice(
                    &bmp.data,
                ))?;
            }
        }
        Ok(buf.into_inner())
    }
}
