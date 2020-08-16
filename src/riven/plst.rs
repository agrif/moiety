use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_u16_table_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TPlst;

impl ResourceType for TPlst {
    type Data = Record<Vec<PictureMeta>>;
    fn name(&self) -> &str {
        "PLST"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PictureMeta {
    pub index: u16,
    pub bitmap_id: u16,
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TPlst, I, Record<Vec<PictureMeta>>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TPlst, input: &mut I)
                   -> Result<Record<Vec<PictureMeta>>>
    {
        Ok(Record(deserialize_u16_table_from(input).await?))
    }
}
