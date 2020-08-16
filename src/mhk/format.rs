use crate::{Format, FormatWrite};

use anyhow::Result;
use smol::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct MhkFormat;

#[async_trait::async_trait(?Send)]
impl<R, I, D> FormatWrite<MhkFormat, R, I, D> for MhkFormat
where
    Self: Format<R, I, D>,
    D: 'static,
    I: AsyncRead + Unpin,
{
    async fn convert(&self, _fmti: &MhkFormat, _res: &R, input: &mut I)
                     -> Result<Vec<u8>>
    {
        let mut data = Vec::new();
        input.read_to_end(&mut data).await?;
        Ok(data)
    }
}
