use crate::{
    filesystem::AsyncRead,
    future::*,
    mhk::{
        deserialize_from,
        deserialize_u16_table_from,
    },
    MhkError,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Event {
    MouseDown,
    MouseStillDown,
    MouseUp,
    MouseEnter,
    MouseWithin,
    MouseLeave,
    LoadCard,
    CloseCard,
    OpenCard,
    DisplayUpdate,
}

pub async fn deserialize_handlers<'a, R>(
    reader: &'a R,
    pos: &'a mut u64,
) -> Result<std::collections::HashMap<Event, Vec<Command>>, MhkError>
where
    R: AsyncRead,
{
    let count: u16 = await!(deserialize_from(reader, pos))?;
    let mut handlers = std::collections::HashMap::with_capacity(count as usize);
    for _ in 0..count {
        let event_type: u16 = await!(deserialize_from(reader, pos))?;
        let commands = await!(deserialize_commands(reader, pos))?;
        let event = match event_type {
            0 => Ok(Event::MouseDown),
            1 => Ok(Event::MouseStillDown),
            2 => Ok(Event::MouseUp),
            3 => Ok(Event::MouseEnter),
            4 => Ok(Event::MouseWithin),
            5 => Ok(Event::MouseLeave),
            6 => Ok(Event::LoadCard),
            7 => Ok(Event::CloseCard),
            // 8 is not seen
            9 => Ok(Event::OpenCard),
            10 => Ok(Event::DisplayUpdate),

            _ => Err(MhkError::InvalidFormat("bad event type")),
        }?;
        handlers.insert(event, commands);
    }
    Ok(handlers)
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InlineSlst {
    pub id: u16,
    pub volume: u16,
    pub balance: u16,
    pub u2: u16,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
pub enum Command {
    DrawBmp {
        tbmp_id: u16,
        left: u16,
        top: u16,
        right: u16,
        bottom: u16,
        u0: u16,
        u1: u16,
        u2: u16,
        u3: u16,
    },
    GotoCard {
        id: u16,
    },
    ActivateInlineSlst {
        sounds: Vec<InlineSlst>,
        fade_flags: u16,
        looping: u16,
        volume: u16,
        u0: u16,
        u1: u16,
    },
    PlayWav {
        id: u16,
        volume: u16,
        u1: u16,
    },
    SetVariable {
        var: u16,
        value: u16,
    },
    Conditional {
        var: u16,
        branches: std::collections::HashMap<u16, Vec<Command>>,
    },
    EnableHotspot {
        hotspot_id: u16,
    },
    DisableHotspot {
        hotspot_id: u16,
    },

    Unknown {
        cmd: u16,
        args: Vec<u16>,
    },
}

// box this one up, because otherwise we make an infinite type
pub fn deserialize_commands<'a, R>(
    reader: &'a R,
    pos: &'a mut u64,
) -> Fut<'a, Result<Vec<Command>, MhkError>>
where
    R: AsyncRead,
{
    use Command::*;
    fut!({
        let count: u16 = await!(deserialize_from(reader, pos))?;
        let mut commands = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let cmd: u16 = await!(deserialize_from(reader, pos))?;
            let args: Vec<u16> =
                await!(deserialize_u16_table_from(reader, pos))?;

            commands.push(match (cmd, args.as_slice()) {
                (1, &[tbmp_id, left, top, right, bottom, u0, u1, u2, u3]) => {
                    Ok(DrawBmp {
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

                (2, &[id]) => Ok(GotoCard { id }),

                (3, slice) => {
                    if slice.len() < 1 {
                        return Err(MhkError::InvalidFormat(
                            "bad inline SLST record",
                        ));
                    }
                    let n = slice[0] as usize;
                    if slice.len() != 6 + 4 * n {
                        return Err(MhkError::InvalidFormat(
                            "bad inline SLST record",
                        ));
                    }
                    let mut sounds = Vec::with_capacity(n);
                    for i in 0..n {
                        sounds.push(InlineSlst {
                            id: slice[1 + i],
                            volume: slice[6 + n + i],
                            balance: slice[6 + 2 * n + i],
                            u2: slice[6 + 3 * n + i],
                        });
                    }

                    Ok(ActivateInlineSlst {
                        sounds,
                        fade_flags: slice[1 + n],
                        looping: slice[2 + n],
                        volume: slice[3 + n],
                        u0: slice[4 + n],
                        u1: slice[5 + n],
                    })
                },

                (4, &[id, volume, u1]) => Ok(PlayWav { id, volume, u1 }),
                (7, &[var, value]) => Ok(SetVariable { var, value }),

                (8, &[var, value_count]) => {
                    let mut branches = std::collections::HashMap::with_capacity(
                        value_count as usize,
                    );
                    for _ in 0..value_count {
                        let value: u16 = await!(deserialize_from(reader, pos))?;
                        let subcommands =
                            await!(deserialize_commands(reader, pos))?;
                        branches.insert(value, subcommands);
                    }
                    Ok(Conditional { var, branches })
                },

                (9, &[hotspot_id]) => Ok(EnableHotspot { hotspot_id }),
                (10, &[hotspot_id]) => Ok(DisableHotspot { hotspot_id }),

                (cmd, args) => {
                    Result::<Command, MhkError>::Ok(Unknown {
                        cmd,
                        args: args.to_owned(),
                    })
                },
            }?);
        }
        Ok(commands)
    })
}
