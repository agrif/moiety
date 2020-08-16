use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, deserialize_from, deserialize_vec_from};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use smol::io::{
    BufReader, AsyncRead, AsyncBufReadExt, AsyncSeek, AsyncSeekExt, SeekFrom,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TName;

impl ResourceType for TName {
    type Data = Record<Vec<Name>>;
    fn name(&self) -> &str {
        "NAME"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Name {
    pub unknown: u16,
    pub name: String,
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TName, I, Record<Vec<Name>>> for MhkFormat
where
    I: AsyncRead + AsyncSeek + Unpin,
{
    async fn parse(&self, _res: &TName, input: &mut I)
                   -> Result<Record<Vec<Name>>>
    {
        let mut bufinput = BufReader::new(input);
        let field_count: u16 = deserialize_from(&mut bufinput).await?;
        let offsets: Vec<u16> = deserialize_vec_from(
            &mut bufinput,
            field_count as usize
        ).await?;
        let values: Vec<u16> = deserialize_vec_from(
            &mut bufinput,
            field_count as usize
        ).await?;
        let mut ret = Vec::with_capacity(offsets.len());
        let start = bufinput.seek(SeekFrom::Current(0)).await?;
        for (offs, val) in offsets.iter().zip(values) {
            let mut name = Vec::new();
            bufinput.seek(SeekFrom::Start(start + *offs as u64)).await?;
            bufinput.read_until(0, &mut name).await?;
            ret.push(Name {
                unknown: val,
                name: String::from_utf8_lossy(&name[..name.len() - 1])
                    .into_owned(),
            });
        }
        Ok(Record(ret))
    }
}
