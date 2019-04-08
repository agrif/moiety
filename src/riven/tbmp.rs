use super::Resource;
use crate::{
    filesystem::AsyncRead,
    future::*,
    mhk::deserialize_from,
    ConvertError,
    FormatFor,
    FormatWriteFor,
    MhkError,
    MhkFormat,
    PngError,
    PngFormat,
};

#[derive(Debug, Clone)]
pub struct Bitmap {
    width: u16,
    height: u16,
    palette: Option<PaletteBitmap>,
    data: Vec<palette::Srgb<u8>>,
}

#[derive(Debug, Clone)]
pub struct PaletteBitmap {
    palette: Vec<palette::Srgb<u8>>,
    image: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BmpHeader {
    width: u16,
    height: u16,
    bytes_per_row: u16,
    compression_flags: u16,
}

#[derive(Debug)]
struct BmpFlags {
    bits_per_pixel: u8,
    bytes_per_pixel: u8,
    palette_present: bool,
    primary: u8,
    secondary: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct BmpPalette {
    table_size: u16,
    bits_per_color: u8,
    color_count: u8,
}

fn bytes_to_rgb(
    bpp: u8,
    bytes: Vec<u8>,
) -> Result<Vec<palette::Srgb<u8>>, MhkError> {
    // remember, riven uses BGR
    match bpp {
        24 => {
            Ok(bytes
                .chunks_exact(3)
                .map(|c| palette::Srgb::new(c[2], c[1], c[0]))
                .collect())
        },
        _ => Err(MhkError::InvalidFormat("unknown bits per pixel")),
    }
}

async fn read_uncompressed<'a, R>(
    header: &'a BmpHeader,
    flags: &'a BmpFlags,
    input: &'a R,
    pos: &'a mut u64,
) -> Result<Vec<u8>, std::io::Error>
where
    R: AsyncRead,
{
    let mut data = vec![
        0;
        header.width as usize
            * header.height as usize
            * flags.bytes_per_pixel as usize
    ];
    for y in 0..header.height as usize {
        let start = y * header.width as usize * flags.bytes_per_pixel as usize;
        let end =
            start + header.width as usize * flags.bytes_per_pixel as usize;
        await!(input.read_exact_at(*pos, &mut data[start..end]))?;
        *pos += header.bytes_per_row as u64;
    }
    Ok(data)
}

impl<R> FormatFor<R, Resource<Bitmap>> for MhkFormat
where
    R: AsyncRead,
{
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Bitmap, MhkError>>
    where
        R: 'a,
    {
        fut!({
            let mut pos = 0;
            let header: BmpHeader = await!(deserialize_from(&input, &mut pos))?;
            let mut flags = BmpFlags {
                bits_per_pixel: match header.compression_flags & 0x7 {
                    0 => 1,
                    1 => 4,
                    2 => 8,
                    3 => 16,
                    4 => 24,
                    _ => return Err(MhkError::InvalidFormat("bad bpp")),
                },
                bytes_per_pixel: 0, // filled in below
                palette_present: (header.compression_flags & (1 << 3)) > 0,
                secondary: ((header.compression_flags & (0xf << 4)) >> 4) as u8,
                primary: ((header.compression_flags & (0xf << 8)) >> 8) as u8,
            };
            flags.bytes_per_pixel = (flags.bits_per_pixel + 7) / 8;

            println!("{:?}", flags);

            // read a palette, if it's here
            let mut palette: Option<Vec<palette::Srgb<u8>>> = None;
            if flags.palette_present || flags.bits_per_pixel == 8 {
                let oldpos = pos;
                let pchunk: BmpPalette =
                    await!(deserialize_from(&input, &mut pos))?;
                let table_size = pchunk.table_size as u64 - (pos - oldpos);
                let mut colors = vec![0; table_size as usize];
                await!(input.read_exact_at(pos, &mut colors))?;

                let colors_parsed =
                    bytes_to_rgb(pchunk.bits_per_color, colors)?;
                if colors_parsed.len() < pchunk.color_count as usize {
                    return Err(MhkError::InvalidFormat(
                        "not enough colors in palette",
                    ));
                }

                palette = Some(colors_parsed);
                pos = oldpos + pchunk.table_size as u64;
            }

            // we don't know how to do any secondary compression
            if flags.secondary != 0 {
                return Err(MhkError::InvalidFormat(
                    "unknown secondary tBMP compression",
                ));
            }

            let data_raw = match flags.primary {
                0 => {
                    await!(read_uncompressed(
                        &header, &flags, &input, &mut pos
                    ))?
                },
                _ => {
                    return Err(MhkError::InvalidFormat(
                        "unknown primary tBMP compression",
                    ))
                },
            };

            if let Some(p) = palette {
                panic!("unhandled palette");
            } else {
                Ok(Bitmap {
                    width: header.width,
                    height: header.height,
                    palette: None,
                    data: bytes_to_rgb(flags.bits_per_pixel, data_raw)?,
                })
            }
        })
    }
}

impl<R> FormatFor<R, Resource<Bitmap>> for PngFormat
where
    R: AsyncRead,
{
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Bitmap, PngError>>
    where
        R: 'a,
    {
        fut!({
            let mut buf = Vec::with_capacity(1 << 16);
            await!(input.read_until_end(&mut buf)).map_err(PngError::Io)?;
            let im = lodepng::decode24(buf).map_err(PngError::Png)?;

            Ok(Bitmap {
                width: im.width as u16,
                height: im.height as u16,
                palette: None,
                data: im
                    .buffer
                    .iter()
                    .map(|p| palette::Srgb::new(p.r, p.g, p.b))
                    .collect(),
            })
        })
    }

    fn extension<'a>(&'a self) -> Option<&'a str> { Some(&".png") }
}

impl<R, Fmt> FormatWriteFor<R, Resource<Bitmap>, Fmt> for PngFormat
where
    R: AsyncRead,
    Fmt: FormatFor<R, Resource<Bitmap>>,
{
    type WriteError = PngError;

    fn write<'a>(
        &'a self,
        input: R,
        fmt: &'a Fmt,
    ) -> Fut<
        'a,
        Result<Vec<u8>, crate::ConvertError<Fmt::Error, Self::WriteError>>,
    >
    where
        R: 'a,
        Fmt: 'a,
    {
        fut!({
            let bmp = await!(fmt.convert(input)).map_err(ConvertError::Read)?;
            // FIXME use palette, if possible
            let buf = lodepng::encode24(
                &bmp.data,
                bmp.width as usize,
                bmp.height as usize,
            )
            .map_err(|e| ConvertError::Write(PngError::Png(e)))?;
            Ok(buf)
        })
    }
}
