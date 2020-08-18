use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_from};
use super::{deserialize_handlers, Command, Event};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::AsyncRead;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct THspt;

impl ResourceType for THspt {
    type Data = Record<Vec<Hotspot>>;
    fn name(&self) -> &str {
        "HSPT"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct Hotspot {
    pub blst_id: u16,
    pub name_rec: i16,
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
    pub u0: u16,
    pub mouse_cursor: u16,
    pub index: u16,
    pub u1: i16,
    pub zip_mode: u16,
    pub script: std::collections::HashMap<Event, Vec<Command>>,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<THspt, I, Record<Vec<Hotspot>>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &THspt, input: &mut I)
                   -> Result<Record<Vec<Hotspot>>>
    {
        // the variable sized script at the end messes up everything
        // so do this the hard way
        let count: u16 = deserialize_from(input).await?;
        let mut ret = Vec::with_capacity(count as usize);
        for _ in 0..count {
            ret.push(Hotspot {
                blst_id: deserialize_from(input).await?,
                name_rec: deserialize_from(input).await?,
                left: deserialize_from(input).await?,
                top: deserialize_from(input).await?,
                right: deserialize_from(input).await?,
                bottom: deserialize_from(input).await?,
                u0: deserialize_from(input).await?,
                mouse_cursor: deserialize_from(input).await?,
                index: deserialize_from(input).await?,
                u1: deserialize_from(input).await?,
                zip_mode: deserialize_from(input).await?,
                script: deserialize_handlers(input).await?,
            });
        }
        Ok(Record(ret))
    }
}
