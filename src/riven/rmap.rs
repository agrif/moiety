use crate::{Record, ResourceType, Format};
use crate::mhk::{MhkFormat, MhkError, deserialize_vec_from};

use anyhow::Result;
use smol::io::{AsyncRead, AsyncReadExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TRmap;

impl ResourceType for TRmap {
    type Data = Record<Vec<u32>>;
    fn name(&self) -> &str {
        "RMAP"
    }
}

#[async_trait::async_trait(?Send)]
impl<I> Format<TRmap, I, Record<Vec<u32>>> for MhkFormat
where
    I: AsyncRead + Unpin,
{
    async fn parse(&self, _res: &TRmap, input: &mut I)
                   -> Result<Record<Vec<u32>>>
    {
        // this one is a bit weird, since it has no prefixed length field
        let mut buf = Vec::with_capacity(100);
        input.read_to_end(&mut buf).await?;
        if buf.len() % 4 != 0 {
            anyhow::bail!(MhkError::InvalidFormat("bad RMAP size"));
        }
        let mut cursor = smol::io::Cursor::new(&buf);
        let res = deserialize_vec_from(&mut cursor, buf.len() / 4).await?;
        Ok(Record(res))
    }
}
