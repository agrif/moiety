use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_u16_table_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TBlst;

impl ResourceType for TBlst {
    type Data = Record<Vec<ButtonMeta>>;
    fn name(&self) -> &str {
        "BLST"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ButtonMeta {
    pub index: u16,
    pub enabled: u16,
    pub hotspot_id: u16,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TBlst, I, Record<Vec<ButtonMeta>>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TBlst, input: &mut I)
                   -> Result<Record<Vec<ButtonMeta>>>
    {
        Ok(Record(deserialize_u16_table_from(input).await?))
    }
}
