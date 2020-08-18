use crate::{Record, ResourceType, Format};
use crate::mhk::{
    MhkFormat,
    deserialize_from, deserialize_vec_from, deserialize_u16_table_from,
};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TSlst;

impl ResourceType for TSlst {
    type Data = Record<Vec<SoundMeta>>;
    fn name(&self) -> &str {
        "SLST"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SoundMeta {
    pub index: u16,
    pub fade_flags: u16,
    pub looping: u16,
    pub global_volume: u16,
    pub u0: u16,
    pub u1: u16,
    pub sounds: Vec<SoundMetaEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SoundMetaEntry {
    pub id: u16,
    pub volume: u16,
    pub balance: i16,
    pub u2: u16,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TSlst, I, Record<Vec<SoundMeta>>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TSlst, input: &mut I)
                   -> Result<Record<Vec<SoundMeta>>>
    {
        let count: u16 = deserialize_from(input).await?;
        let mut ret = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let index = deserialize_from(input).await?;
            let sound_ids: Vec<u16> = deserialize_u16_table_from(input).await?;
            let fade_flags = deserialize_from(input).await?;
            let looping = deserialize_from(input).await?;
            let global_volume = deserialize_from(input).await?;
            let u0 = deserialize_from(input).await?;
            let u1 = deserialize_from(input).await?;
            let volumes: Vec<u16> = deserialize_vec_from(
                input, sound_ids.len()).await?;
            let balances: Vec<i16> = deserialize_vec_from(
                input, sound_ids.len()).await?;
            let u2: Vec<u16> = deserialize_vec_from(
                input, sound_ids.len()).await?;

            let sounds = sound_ids.iter().enumerate().map(|(i, id)| {
                SoundMetaEntry {
                    id: *id,
                    volume: volumes[i],
                    balance: balances[i],
                    u2: u2[i],
                }
            }).collect();
            ret.push(SoundMeta {
                index,
                fade_flags,
                looping,
                global_volume,
                u0,
                u1,
                sounds,
            });
        }
        Ok(Record(ret))
    }
}
