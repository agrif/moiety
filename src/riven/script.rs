use crate::mhk::{MhkError, deserialize_from, deserialize_u16_table_from};

use std::pin::Pin;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

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

pub async fn deserialize_handlers<R>(
    reader: &mut R,
) -> Result<std::collections::HashMap<Event, Vec<Command>>>
where
    R: AsyncRead + Unpin,
{
    let count: u16 = deserialize_from(reader).await?;
    let mut handlers = std::collections::HashMap::with_capacity(count as usize);
    for _ in 0..count {
        let event_type: u16 = deserialize_from(reader).await?;
        let commands = deserialize_commands(reader).await?;
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
#[serde(rename_all = "kebab-case")]
pub enum TransitionDirection {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionCode {
    Direction {
        direction: TransitionDirection,
        new_move: bool,
        old_move: bool,
    },
    Blend,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
    SetCursor {
        cursor: u16,
    },
    Pause {
        ms: u16,
        u0: u16,
    },
    Call {
        cmd: u16,
        args: Vec<u16>,
    },
    Transition {
        code: TransitionCode,
        // rarely seen, left top right bottom
        rect: Option<(u16, u16, u16, u16)>,
    },
    ReloadCard,
    DisableScreenUpdate,
    EnableScreenUpdate,
    IncrementVariable {
        var: u16,
        value: u16,
    },
    GotoStack {
        stack_name: u16,
        code: u32,
    },
    PlayForegroundMovie {
        code: u16,
    },
    PlayBackgroundMovie {
        code: u16,
    },
    ActivatePlst {
        record: u16,
    },
    ActivateSlst {
        record: u16,
    },
    ActivateBlst {
        record: u16,
    },
    ActivateFlst {
        record: u16,
    },
    ZipMode,
    ActivateMlst {
        record: u16,
        u0: u16,
    },

    Unknown {
        cmd: u16,
        args: Vec<u16>,
    },
}

// box this one up, because otherwise we make an infinite type
pub fn deserialize_commands<'a, R>(
    reader: &'a mut R,
) -> Pin<Box<dyn smol::future::Future<Output=Result<Vec<Command>>> + 'a>>
where
    R: AsyncRead + Unpin,
{
    use Command::*;

    Box::pin(async move {
        let count: u16 = deserialize_from(reader).await?;
        let mut commands = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let cmd: u16 = deserialize_from(reader).await?;
            let args: Vec<u16> =
                deserialize_u16_table_from(reader).await?;
            
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
                        anyhow::bail!(MhkError::InvalidFormat(
                            "bad inline SLST record",
                        ));
                    }
                    let n = slice[0] as usize;
                    if slice.len() != 6 + 4 * n {
                        anyhow::bail!(MhkError::InvalidFormat(
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
                        let value: u16 = deserialize_from(reader).await?;
                        let subcommands =
                            deserialize_commands(reader).await?;
                        branches.insert(value, subcommands);
                    }
                    Ok(Conditional { var, branches })
                },

                (9, &[hotspot_id]) => Ok(EnableHotspot { hotspot_id }),
                (10, &[hotspot_id]) => Ok(DisableHotspot { hotspot_id }),

                (12, &[u0]) => {
                    Ok(Unknown {
                        cmd,
                        args: vec![u0],
                    })
                },
                (13, &[cursor]) => Ok(SetCursor { cursor }),
                (14, &[ms, u0]) => Ok(Pause { ms, u0 }),

                (17, args) => {
                    if args.len() < 2 || args.len() < 2 + args[1] as usize {
                        anyhow::bail!(MhkError::InvalidFormat("bad call"));
                    }
                    Ok(Call {
                        cmd: args[0],
                        args: args[2..2 + args[1] as usize].to_owned(),
                    })
                },

                (18, args) => {
                    if args.len() != 1 && args.len() != 5 {
                        anyhow::bail!(MhkError::InvalidFormat("bad transition"));
                    }
                    let mut rect = None;
                    if args.len() == 5 {
                        rect = Some((args[1], args[2], args[3], args[4]));
                    }
                    let codenum = args[0];
                    let code = if codenum >= 16 {
                        TransitionCode::Blend
                    } else {
                        TransitionCode::Direction {
                            direction: match codenum & 0x3 {
                                0 => TransitionDirection::Left,
                                1 => TransitionDirection::Right,
                                2 => TransitionDirection::Top,
                                3 => TransitionDirection::Bottom,
                                _ => unreachable!(),
                            },
                            new_move: (codenum & 0x4) > 0,
                            old_move: (codenum & 0x8) > 0,
                        }
                    };
                    Ok(Transition { code, rect })
                },

                (19, &[]) => Ok(ReloadCard),
                (20, &[]) => Ok(DisableScreenUpdate),
                (21, &[]) => Ok(EnableScreenUpdate),

                (24, &[var, value]) => Ok(IncrementVariable { var, value }),

                (27, &[stack_name, code_hi, code_lo]) => {
                    Ok(GotoStack {
                        stack_name,
                        code: ((code_hi as u32) << 16) | (code_lo as u32),
                    })
                },

                (28, &[code]) => {
                    Ok(Unknown {
                        cmd,
                        args: vec![code],
                    })
                },
                (29, &[]) => Ok(Unknown { cmd, args: vec![] }),

                (31, &[code]) => {
                    Ok(Unknown {
                        cmd,
                        args: vec![code],
                    })
                },
                (32, &[code]) => Ok(PlayForegroundMovie { code }),
                (33, &[code]) => Ok(PlayBackgroundMovie { code }),
                (34, &[u0]) => {
                    Ok(Unknown {
                        cmd,
                        args: vec![u0],
                    })
                },

                (36, &[]) => Ok(Unknown { cmd, args: vec![] }),
                (37, &[]) => Ok(Unknown { cmd, args: vec![] }),

                (38, &[u1, u2, u3, u4, u5]) => {
                    Ok(Unknown {
                        cmd,
                        args: vec![u1, u2, u3, u4, u5],
                    })
                },

                (39, &[record]) => Ok(ActivatePlst { record }),
                (40, &[record]) => Ok(ActivateSlst { record }),
                (41, &[u0]) => {
                    Ok(Unknown {
                        cmd,
                        args: vec![u0],
                    })
                },

                (43, &[record]) => Ok(ActivateBlst { record }),
                (44, &[record]) => Ok(ActivateFlst { record }),
                (45, &[]) => Ok(ZipMode),
                (46, &[record, u0]) => Ok(ActivateMlst { record, u0 }),

                (_cmd, _args) => {
                    Err(MhkError::InvalidFormat("unknown script command"))
                    // Result::<Command, MhkError>::Ok(Unknown {
                    //    cmd,
                    //    args: args.to_owned(),
                    //})
                },
            }?);
        }
        Ok(commands)
    })
}
