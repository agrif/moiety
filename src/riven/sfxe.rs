use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, MhkError, deserialize_from, deserialize_vec_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::{AsyncRead, AsyncSeek, AsyncSeekExt, SeekFrom};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TSfxe;

impl ResourceType for TSfxe {
    type Data = Record<Effect>;
    fn name(&self) -> &str {
        "SFXE"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Effect {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub effect_speed: u16,
    pub u0: u16,
    pub u1: u16,
    pub u2: u32,
    pub u3: u32,
    pub u4: u32,
    pub u5: u32,
    pub u6: u32,
    pub u7: Vec<u8>,
    pub frames: Vec<Vec<EffectCommand>>,
}

// SFXE resources can get long, so be careful with command size
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum EffectCommand {
    #[serde(rename = "inc")]
    IncrementRow {
        #[serde(rename = "n")]
        amount: u16,
    },
    #[serde(rename = "copy")]
    Copy {
        #[serde(rename = "dl")]
        dst_left: u16,
        #[serde(rename = "sl")]
        src_left: u16,
        #[serde(rename = "st")]
        src_top: u16,
        #[serde(rename = "n")]
        row_width: u16,
    },
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TSfxe, I, Record<Effect>> for MhkFormat
where
    I: AsyncRead + AsyncSeek + Unpin,
{
    async fn parse(&self, _res: &TSfxe, input: &mut I)
                   -> Result<Record<Effect>>
    {
        let magic: [u8; 2] = deserialize_from(input).await?;
        if magic != "SL".as_bytes() {
            anyhow::bail!(MhkError::InvalidFormat("bad SFXE signature"));
        }

        let frame_count: u16 = deserialize_from(input).await?;
        let offset_table_position: u32 = deserialize_from(input).await?;

        let left = deserialize_from(input).await?;
        let top = deserialize_from(input).await?;
        let right = deserialize_from(input).await?;
        let bottom = deserialize_from(input).await?;
        let effect_speed = deserialize_from(input).await?;
        let u0 = deserialize_from(input).await?;

        let alt_top: u16 = deserialize_from(input).await?;
        let alt_left: u16 = deserialize_from(input).await?;
        let alt_bottom: u16 = deserialize_from(input).await?;
        let alt_right: u16 = deserialize_from(input).await?;
        if alt_top != top || alt_left != left
            || alt_bottom != bottom || alt_right != right
        {
            anyhow::bail!(MhkError::InvalidFormat("bad SFXE resource"));
        }

        let u1 = deserialize_from(input).await?;

        let alt_frame_count: u16 = deserialize_from(input).await?;
        if alt_frame_count != frame_count {
            anyhow::bail!(MhkError::InvalidFormat("bad SFXE resource"));
        }

        let u2 = deserialize_from(input).await?;
        let u3 = deserialize_from(input).await?;
        let u4 = deserialize_from(input).await?;
        let u5 = deserialize_from(input).await?;
        let u6 = deserialize_from(input).await?;

        // header is 52 bytes exactly, so unknown section is...
        if offset_table_position < 52 {
            anyhow::bail!(MhkError::InvalidFormat("bad SFXE resource"));
        }
        let unknown_size = offset_table_position - 52;
        let u7 = deserialize_vec_from(input, unknown_size as usize).await?;

        // we are now positioned at offset_table_position, but to be sure
        input.seek(SeekFrom::Start(offset_table_position as u64)).await?;
        let offset_table: Vec<u32> = deserialize_vec_from(
            input, frame_count as usize).await?;
        let mut frames = Vec::with_capacity(frame_count as usize);
        for offset in offset_table {
            input.seek(SeekFrom::Start(offset as u64)).await?;
            let mut frame = Vec::with_capacity(10);
            loop {
                let cmd: u16 = deserialize_from(input).await?;
                match cmd {
                    1 => {
                        // increment row
                        // there can be a *lot* of these, so we do some
                        // run-length encoding in our representation
                        let len = frame.len();
                        if len == 0 {
                            frame.push(EffectCommand::IncrementRow {
                                amount: 1,
                            });
                            continue;
                        }
                        match frame[len - 1] {
                            EffectCommand::IncrementRow { ref mut amount } => {
                                *amount += 1;
                            },
                            _ => {
                                frame.push(EffectCommand::IncrementRow {
                                    amount: 1,
                                });
                            }
                        }
                    },
                    3 => {
                        // copy
                        frame.push(EffectCommand::Copy {
                            dst_left: deserialize_from(input).await?,
                            src_left: deserialize_from(input).await?,
                            src_top: deserialize_from(input).await?,
                            row_width: deserialize_from(input).await?,
                        });
                    },
                    4 => {
                        // end of commands
                        break;
                    }
                    _ => anyhow::bail!(
                        MhkError::InvalidFormat("bad SFXE command")),
                }
            }
            frames.push(frame);
        }
        
        Ok(Record(Effect {
            left,
            top,
            right,
            bottom,
            effect_speed,
            u0,
            u1,
            u2,
            u3,
            u4,
            u5,
            u6,
            u7,
            frames,
        }))
    }
}
