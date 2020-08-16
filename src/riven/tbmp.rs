use crate::{Bitmap, PaletteBitmap, ResourceType, Format};
use crate::mhk::{MhkFormat, MhkError, deserialize_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TBmp;

impl ResourceType for TBmp {
    type Data = Bitmap;
    fn name(&self) -> &str {
        "tBMP"
    }
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
) -> Result<Vec<palette::Srgb<u8>>> {
    // remember, riven uses BGR
    match bpp {
        24 => {
            Ok(bytes
                .chunks_exact(3)
                .map(|c| palette::Srgb::new(c[2], c[1], c[0]))
                .collect())
        },
        _ => anyhow::bail!(MhkError::InvalidFormat("unknown bits per pixel")),
    }
}

async fn read_uncompressed<R>(
    header: &BmpHeader,
    flags: &BmpFlags,
    input: &mut R,
) -> Result<Vec<u8>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    let mut data = vec![
        0;
        header.width as usize
            * header.height as usize
            * flags.bytes_per_pixel as usize
    ];
    let jump = header.bytes_per_row as i64
        - (header.width as i64 * flags.bytes_per_pixel as i64);
    for y in 0..header.height as usize {
        let start = y * header.width as usize * flags.bytes_per_pixel as usize;
        let end =
            start + header.width as usize * flags.bytes_per_pixel as usize;
        input.read_exact(&mut data[start..end]).await?;
        input.seek(SeekFrom::Current(jump)).await?;
    }
    Ok(data)
}

fn copy_lookback_repeat(
    data: &mut Vec<u8>,
    lookback: usize,
    len: usize,
) -> Option<()> {
    if data.len() < lookback || lookback == 0 {
        return None;
    }

    let start = data.len() - lookback;
    let end = start + len;
    data.reserve(len);
    unsafe {
        // we will initialize this immediately after...
        data.set_len(data.len() + len);

        for i in start..end {
            data[i + lookback] = data[i];
        }
    }

    Some(())
}

async fn read_riven<R>(
    header: &BmpHeader,
    flags: &BmpFlags,
    input: &mut R,
) -> Result<Vec<u8>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    if flags.bits_per_pixel != 8 {
        anyhow::bail!(MhkError::InvalidFormat(
            "bad bits per pixel in tBMP riven compression",
        ));
    }

    let mut cmds =
        Vec::with_capacity(header.width as usize * header.height as usize);
    let mut out = Vec::with_capacity(
        header.height as usize * header.bytes_per_row as usize,
    );
    input.read_to_end(&mut cmds).await?;

    // used for when something unexpected comes up
    let invalid_err = || MhkError::InvalidFormat("bad tBMP riven command");

    // the first 4 bytes of the command stream are unknown, so...
    let mut c = 4;
    'decode: while c < cmds.len() {
        match cmds[c] {
            0x00 => {
                // end of stream
                break 'decode;
            },
            n @ 0x01..=0x3f => {
                // output n pixel duplets, direct from stream
                c += 1;
                out.extend_from_slice(
                    cmds.get(c..c + 2 * n as usize).ok_or_else(invalid_err)?,
                );
                c += 2 * n as usize;
            },
            mut n @ 0x40..=0x7f => {
                // repeat last 2 pixels n times, where..=
                n &= 0x3f;
                c += 1;
                copy_lookback_repeat(&mut out, 2, 2 * n as usize)
                    .ok_or_else(invalid_err)?;
            },
            mut n @ 0x80..=0xbf => {
                // repeat last 4 pixels n times, where..=
                n &= 0x3f;
                c += 1;
                copy_lookback_repeat(&mut out, 4, 4 * n as usize)
                    .ok_or_else(invalid_err)?;
            },
            mut n_subcommands @ 0xc0..=0xff => {
                // n_subcommands follow, where..=
                n_subcommands &= 0x3f;
                c += 1;
                while c < cmds.len() && n_subcommands > 0 {
                    n_subcommands -= 1;
                    match cmds[c..cmds.len().min(c + 4)] {
                        [] | [0x00, ..] => {
                            // end of stream
                            break 'decode;
                        },
                        [mut m @ 0x01..=0x0f, ..] => {
                            // repeat duplet at relative position m, where..=
                            m &= 0x0f;
                            c += 1;
                            let start = out.len() - 2 * m as usize;
                            let duplet = out
                                .get(start..start + 2)
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[a, b]);
                        },
                        [0x10, p, ..] => {
                            // repeat last duplet, but change second pixel to p
                            c += 2;
                            let &a = out
                                .get(out.len() - 2)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[a, p]);
                        },
                        [mut m @ 0x11..=0x1f, ..] => {
                            // output first pixel of last duplet, then pixel at -m
                            // (-m is given in pixels)
                            // careful: -m is relative to the second
                            // output pixel!
                            m &= 0x0f;
                            c += 1;
                            out.reserve(2);
                            let &a = out
                                .get(out.len() - 2)
                                .ok_or_else(invalid_err)?;
                            out.push(a);
                            let &b = out
                                .get(out.len() - m as usize)
                                .ok_or_else(invalid_err)?;
                            out.push(b);
                        },
                        [mut x @ 0x20..=0x2f, ..] => {
                            // repeat last duplet, but add x to second
                            x &= 0x0f;
                            c += 1;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[a, b.wrapping_add(x)]);
                        },
                        [mut x @ 0x30..=0x3f, ..] => {
                            // repeat last duplet, but subtract x from second
                            x &= 0x0f;
                            c += 1;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[a, b.wrapping_sub(x)]);
                        },
                        [0x40, p, ..] => {
                            // repeat last duplet, but change first pixel to p
                            c += 2;
                            let &b = out
                                .get(out.len() - 1)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[p, b]);
                        },
                        [mut m @ 0x41..=0x4f, ..] => {
                            // output pixel at -m, then second pixel of last duplet
                            // (-m is given in pixels)
                            m &= 0x0f;
                            c += 1;
                            let &a = out
                                .get(out.len() - m as usize)
                                .ok_or_else(invalid_err)?;
                            let &b = out
                                .get(out.len() - 1)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[a, b]);
                        },
                        [0x50, p1, p2, ..] => {
                            // output two pixels directly
                            c += 3;
                            out.extend_from_slice(&[p1, p2]);
                        },
                        [mut m @ 0x51..=0x57, p, ..] => {
                            // output pixel at -m, then p
                            m &= 0x07;
                            c += 2;
                            let &a = out
                                .get(out.len() - m as usize)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[a, p]);
                        },
                        // 0x58 is intentionally missing
                        [mut m @ 0x59..=0x5f, p, ..] => {
                            // output p, then pixel at -m
                            // careful: -m is relative to the second
                            // output pixel!
                            m &= 0x07;
                            c += 2;
                            out.reserve(2);
                            out.push(p);
                            let &b = out
                                .get(out.len() - m as usize)
                                .ok_or_else(invalid_err)?;
                            out.push(b);
                        },
                        [mut x @ 0x60..=0x6f, p, ..] => {
                            // output p, then (second pixel last duplet) + x
                            x &= 0x0f;
                            c += 2;
                            let &b = out
                                .get(out.len() - 1)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[p, b.wrapping_add(x)]);
                        },
                        [mut x @ 0x70..=0x7f, p, ..] => {
                            // output p, then (second pixel last duplet) - x
                            x &= 0x0f;
                            c += 2;
                            let &b = out
                                .get(out.len() - 1)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[p, b.wrapping_sub(x)]);
                        },
                        [mut x @ 0x80..=0x8f, ..] => {
                            // repeat last duplet, but add x to first
                            x &= 0x0f;
                            c += 1;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[a.wrapping_add(x), b]);
                        },
                        [mut x @ 0x90..=0x9f, p, ..] => {
                            // output (first pixel last duplet) + x, then p
                            x &= 0x0f;
                            c += 2;
                            let &a = out
                                .get(out.len() - 2)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[a.wrapping_add(x), p]);
                        },
                        [0xa0, xy, ..] => {
                            // repeat last duplet, (+x, +y)
                            let x = (xy & 0xf0) >> 4;
                            let y = xy & 0x0f;
                            c += 2;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[
                                a.wrapping_add(x),
                                b.wrapping_add(y),
                            ]);
                        },
                        [0xb0, xy, ..] => {
                            // repeat last duplet, (+x, -y)
                            let x = (xy & 0xf0) >> 4;
                            let y = xy & 0x0f;
                            c += 2;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[
                                a.wrapping_add(x),
                                b.wrapping_sub(y),
                            ]);
                        },
                        [mut x @ 0xc0..=0xcf, ..] => {
                            // repeat last duplet, but subtract x from first
                            x &= 0x0f;
                            c += 1;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[a.wrapping_sub(x), b]);
                        },
                        [mut x @ 0xd0..=0xdf, p, ..] => {
                            // output (first pixel last duplet) - x, then p
                            x &= 0x0f;
                            c += 2;
                            let &a = out
                                .get(out.len() - 2)
                                .ok_or_else(invalid_err)?;
                            out.extend_from_slice(&[a.wrapping_sub(x), p]);
                        },
                        [0xe0, xy, ..] => {
                            // repeat last duplet, (-x, +y)
                            let x = (xy & 0xf0) >> 4;
                            let y = xy & 0x0f;
                            c += 2;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[
                                a.wrapping_sub(x),
                                b.wrapping_add(y),
                            ]);
                        },
                        [0xf0, xy, ..] | [0xff, xy, ..] => {
                            // repeat last duplet, (-x, -y)
                            let x = (xy & 0xf0) >> 4;
                            let y = xy & 0x0f;
                            c += 2;
                            let duplet = out
                                .get(out.len() - 2..out.len())
                                .ok_or_else(invalid_err)?;
                            let (a, b) = (duplet[0], duplet[1]);
                            out.extend_from_slice(&[
                                a.wrapping_sub(x),
                                b.wrapping_sub(y),
                            ]);
                        },
                        [0xfc, nrm, mlow, ..] => {
                            c += 3;
                            let n = (nrm & 0xf8) >> 3;
                            let r = (nrm & 0x4) >> 2;
                            let mut m = mlow as usize;
                            m |= (nrm as usize & 0x3) << 8;

                            // repeat n+2 duplets from pixel -m
                            // if r is 0, another byte follows and
                            // this is used for the last pixel
                            copy_lookback_repeat(
                                &mut out,
                                m,
                                2 * (n as usize + 2),
                            )
                            .ok_or_else(invalid_err)?;
                            if r == 0 {
                                let last =
                                    *cmds.get(c).ok_or_else(invalid_err)?;
                                c += 1;
                                // this must exist, as we extend above by design
                                let last_pos = out.len() - 1;
                                out[last_pos] = last;
                            }
                        },
                        [cmd, ..] => {
                            // what remains are ugly repeat commands
                            if cmd & 0xa0 != 0xa0 || cmd & 0x0c == 0 {
                                // this is note one of them
                                anyhow::bail!(MhkError::InvalidFormat(
                                    "unknown tBMP riven subcommand",
                                ));
                            }

                            // decode x
                            let x = ((cmd & 0x40) >> 3) | ((cmd & 0x1c) >> 2);
                            // remove the values that end ..00, and renumber
                            let xskip = x - ((x + 3) / 4);
                            // figure out r and n
                            let r = xskip & 0x1;
                            let n = (xskip >> 1) + 2;
                            c += 1;

                            // read m
                            let mut m =
                                *cmds.get(c).ok_or_else(invalid_err)? as usize;
                            m |= (cmd as usize & 0x03) << 8;
                            c += 1;

                            // repeat n duplets from pixel -m
                            // if r is 0, then another byte follows and
                            // the last pixel is set to that value.
                            copy_lookback_repeat(&mut out, m, 2 * n as usize)
                                .ok_or_else(invalid_err)?;
                            if r == 0 {
                                let last =
                                    *cmds.get(c).ok_or_else(invalid_err)?;
                                c += 1;
                                // this must exist, as n > 2 by design
                                let last_pos = out.len() - 1;
                                out[last_pos] = last;
                            }
                        },
                    }
                }
            },
        }
    }

    if header.height > 0 {
        for y in 1..header.height as usize {
            let from = y * header.bytes_per_row as usize;
            if from >= out.len() {
                break;
            }
            let to = y * header.width as usize;
            let from_end = (from + header.width as usize).min(out.len());
            out.copy_within(from..from_end, to);
        }
    }

    out.resize(header.width as usize * header.height as usize, 0);
    Ok(out)
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TBmp, I, Bitmap> for MhkFormat
where
    I: AsyncRead + AsyncSeek + Unpin,
{
    async fn parse(&self, _res: &TBmp, input: &mut I) -> Result<Bitmap> {
        let header: BmpHeader = deserialize_from(input).await?;
        let mut flags = BmpFlags {
            bits_per_pixel: match header.compression_flags & 0x7 {
                0 => 1,
                1 => 4,
                2 => 8,
                3 => 16,
                4 => 24,
                _ => anyhow::bail!(MhkError::InvalidFormat("bad bpp")),
            },
            bytes_per_pixel: 0, // filled in below
            palette_present: (header.compression_flags & (1 << 3)) > 0,
            secondary: ((header.compression_flags & (0xf << 4)) >> 4) as u8,
            primary: ((header.compression_flags & (0xf << 8)) >> 8) as u8,
        };
        flags.bytes_per_pixel = (flags.bits_per_pixel + 7) / 8;

        if header.bytes_per_row < header.width {
            anyhow::bail!(MhkError::InvalidFormat("bad tBMP stride"));
        }

        // read a palette, if it's here
        let mut palette: Option<Vec<palette::Srgb<u8>>> = None;
        if flags.palette_present || flags.bits_per_pixel == 8 {
            let oldpos = input.seek(SeekFrom::Current(0)).await?;
            let pchunk: BmpPalette = deserialize_from(input).await?;
            let newpos = input.seek(SeekFrom::Current(0)).await?;
            let table_size = pchunk.table_size as u64 - (newpos - oldpos);
            let mut colors = vec![0; table_size as usize];
            input.read_exact(&mut colors).await?;

            let colors_parsed = bytes_to_rgb(pchunk.bits_per_color, colors)?;
            if colors_parsed.len() < pchunk.color_count as usize {
                anyhow::bail!(MhkError::InvalidFormat(
                    "not enough colors in palette",
                ));
            }

            palette = Some(colors_parsed);
            input.seek(SeekFrom::Start(oldpos + pchunk.table_size as u64))
                .await?;
        }

        // we don't know how to do any secondary compression
        if flags.secondary != 0 {
            anyhow::bail!(MhkError::InvalidFormat(
                "unknown secondary tBMP compression",
            ));
        }

        let data_raw = match flags.primary {
            0 => {
                read_uncompressed(&header, &flags, input).await?
            },
            4 => read_riven(&header, &flags, input).await?,
            _ => {
                anyhow::bail!(MhkError::InvalidFormat(
                    "unknown primary tBMP compression",
                ))
            },
        };

        if let Some(p) = palette {
            let colored = data_raw
                .iter()
                .map(|&b| {
                    p.get(b as usize)
                        .cloned()
                        .unwrap_or(palette::Srgb::new(0, 0, 0))
                })
                .collect();
            Ok(Bitmap {
                width: header.width,
                height: header.height,
                palette: Some(PaletteBitmap {
                    palette: p,
                    image: data_raw,
                }),
                data: colored,
            })
        } else {
            Ok(Bitmap {
                width: header.width,
                height: header.height,
                palette: None,
                data: bytes_to_rgb(flags.bits_per_pixel, data_raw)?,
            })
        }
    }
}

