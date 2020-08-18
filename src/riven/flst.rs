use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_u16_table_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TFlst;

impl ResourceType for TFlst {
    type Data = Record<Vec<EffectMeta>>;
    fn name(&self) -> &str {
        "FLST"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EffectMeta {
    pub index: u16,
    pub sfxe_id: u16,
    pub u0: u16,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TFlst, I, Record<Vec<EffectMeta>>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TFlst, input: &mut I)
                   -> Result<Record<Vec<EffectMeta>>>
    {
        Ok(Record(deserialize_u16_table_from(input).await?))
    }
}
