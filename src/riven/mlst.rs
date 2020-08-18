use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_u16_table_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TMlst;

impl ResourceType for TMlst {
    type Data = Record<Vec<MovieMeta>>;
    fn name(&self) -> &str {
        "MLST"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MovieMeta {
    pub index: u16,
    pub movie_id: u16,
    pub code: u16,
    pub left: u16,
    pub top: u16,
    pub u0: [u16; 3],
    pub looping: u16,
    pub volume: u16,
    pub u1: u16,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TMlst, I, Record<Vec<MovieMeta>>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TMlst, input: &mut I)
                   -> Result<Record<Vec<MovieMeta>>>
    {
        Ok(Record(deserialize_u16_table_from(input).await?))
    }
}
