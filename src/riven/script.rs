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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
pub enum Command {
    DrawBMP {
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
    Conditional {
        var: u16,
        branches: std::collections::HashMap<u16, Vec<Command>>,
    },

    Dummy,
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

// box this one up, because otherwise we make an infinite type
pub fn deserialize_commands<'a, R>(
    reader: &'a R,
    pos: &'a mut u64,
) -> Fut<'a, Result<Vec<Command>, MhkError>>
where
    R: AsyncRead,
{
    fut!({
        let count: u16 = await!(deserialize_from(reader, pos))?;
        let mut commands = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let cmd: u16 = await!(deserialize_from(reader, pos))?;
            let args: Vec<u16> =
                await!(deserialize_u16_table_from(reader, pos))?;

            commands.push(match (cmd, args.as_slice()) {
                (1, &[tbmp_id, left, top, right, bottom, u0, u1, u2, u3]) => {
                    Ok(Command::DrawBMP {
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
                (2, &[id]) => Ok(Command::GotoCard { id }),
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
                    Ok(Command::Conditional { var, branches })
                },

                _ => Result::<Command, MhkError>::Ok(Command::Dummy),
            }?);
        }
        Ok(commands)
    })
}
