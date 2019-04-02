use crate::future::*;
use super::MhkError;
use super::utility::*;

#[derive(Debug)]
pub struct MhkFormat;

impl<R> crate::Format<R> for MhkFormat where R: crate::AsyncRead {
    type Error = MhkError;
}

// temporary
impl<R> crate::FormatFor<R, crate::Riven<crate::Card>> for MhkFormat where R: crate::AsyncRead {
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<crate::Card, MhkError>> where R: 'a {
        fut!({
            let mut pos = 0;
            let name_rec = await!(deserialize_from(&input, &mut pos))?;
            let zip_mode_place = await!(deserialize_from(&input, &mut pos))?;
            let script = await!(deserialize_handlers(&input, &mut pos))?;
            Ok(crate::Card {
                name_rec,
                zip_mode_place,
                script,
            })
        })
    }
}

async fn deserialize_handlers<'a, R>(reader: &'a R, pos: &'a mut u64) -> Result<std::collections::HashMap<crate::Event, Vec<crate::Command>>, MhkError> where R: crate::AsyncRead {
    let count: u16 = await!(deserialize_from(reader, pos))?;
    let mut handlers = std::collections::HashMap::with_capacity(count as usize);
    for _ in 0..count {
        let event_type: u16 = await!(deserialize_from(reader, pos))?;
        let commands = await!(deserialize_commands(reader, pos))?;
        let event = match event_type {
            0  => Ok(crate::Event::MouseDown),
            1  => Ok(crate::Event::MouseStillDown),
            2  => Ok(crate::Event::MouseUp),
            3  => Ok(crate::Event::MouseEnter),
            4  => Ok(crate::Event::MouseWithin),
            5  => Ok(crate::Event::MouseLeave),
            6  => Ok(crate::Event::LoadCard),
            7  => Ok(crate::Event::CloseCard),
            // 8 is not seen
            9  => Ok(crate::Event::OpenCard),
            10 => Ok(crate::Event::DisplayUpdate),

            _  => Err(MhkError::InvalidFormat("bad event type")),
        }?;
        handlers.insert(event, commands);
    }
    Ok(handlers)
}

// box this one up, because otherwise we make an infinite type
fn deserialize_commands<'a, R>(reader: &'a R, pos: &'a mut u64) -> Fut<'a, Result<Vec<crate::Command>, MhkError>> where R: crate::AsyncRead {
    fut!({
        let count: u16 = await!(deserialize_from(reader, pos))?;
        let mut commands = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let cmd: u16 = await!(deserialize_from(reader, pos))?;
            let args: Vec<u16> = await!(deserialize_u16_table_from(reader, pos))?;

            commands.push(match (cmd, args.as_slice()) {
                (1, &[tbmp_id, left, top, right, bottom, u0, u1, u2, u3]) => {
                    Ok(crate::Command::DrawBMP {
                        tbmp_id,
                        left,
                        top,
                        right,
                        bottom,
                        u0,
                        u1,
                        u2,
                        u3,
                    })
                },
                (2, &[id]) => {
                    Ok(crate::Command::GotoCard {
                        id,
                    })
                },
                (8, &[var, value_count]) => {
                    let mut branches = std::collections::HashMap::with_capacity(value_count as usize);
                    for _ in 0..value_count {
                        let value: u16 = await!(deserialize_from(reader, pos))?;
                        let subcommands = await!(deserialize_commands(reader, pos))?;
                        branches.insert(value, subcommands);
                    }
                    Ok(crate::Command::Conditional {
                        var,
                        branches,
                    })
                },

                _ => Result::<crate::Command, MhkError>::Ok(crate::Command::Dummy),
            }?);
        }
        Ok(commands)        
    })
}

impl<R> crate::FormatFor<R, crate::Riven<Vec<crate::ButtonMeta>>> for MhkFormat where R: crate::AsyncRead {
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Vec<crate::ButtonMeta>, MhkError>> where R: 'a {
        fut!({
            let mut pos = 0;
            await!(deserialize_u16_table_from(&input, &mut pos))
        })
    }
}

impl<R> crate::FormatFor<R, crate::Riven<Vec<crate::Name>>> for MhkFormat where R: crate::AsyncRead {
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Vec<crate::Name>, MhkError>> where R: 'a {
        fut!({
            let mut pos = 0;
            let field_count: u16 = await!(deserialize_from(&input, &mut pos))?;
            let offsets: Vec<u16> = await!(deserialize_vec_from(&input, &mut pos, field_count as usize))?;
            let values: Vec<u16> = await!(deserialize_vec_from(&input, &mut pos, field_count as usize))?;
            let mut ret = Vec::with_capacity(offsets.len());
            for (offs, val) in offsets.iter().zip(values) {
                let mut name = Vec::new();
                await!(input.read_until_at(pos + *offs as u64, 0, &mut name))?;
                ret.push(crate::Name {
                    unknown: val,
                    name: String::from_utf8_lossy(&name[..name.len()-1]).into_owned(),
                });
            }
            Ok(ret)
        })
    }
}

impl<R> crate::FormatFor<R, crate::Riven<Vec<crate::PictureMeta>>> for MhkFormat where R: crate::AsyncRead {
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Vec<crate::PictureMeta>, MhkError>> where R: 'a {
        fut!({
            let mut pos = 0;
            await!(deserialize_u16_table_from(&input, &mut pos))
        })
    }
}

