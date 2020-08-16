use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_from};
use super::{deserialize_handlers, Command, Event};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TCard;

impl ResourceType for TCard {
    type Data = Record<Card>;
    fn name(&self) -> &str {
        "CARD"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct Card {
    pub name_rec: i16,
    pub zip_mode_place: u16,
    pub script: std::collections::HashMap<Event, Vec<Command>>,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TCard, I, Record<Card>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TCard, input: &mut I) -> Result<Record<Card>>
    {
        let name_rec = deserialize_from(input).await?;
        let zip_mode_place = deserialize_from(input).await?;
        let script = deserialize_handlers(input).await?;
        Ok(Record(Card {
            name_rec,
            zip_mode_place,
            script,
        }))
    }
}
