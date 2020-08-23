use anyhow::Result;

use smol::io::{AsyncRead, AsyncSeek};

#[derive(Debug)]
pub struct ZArchive<R>(unshield::AsyncArchive<R>);

impl<R> ZArchive<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    pub async fn new(inner: R) -> Result<Self> {
        Ok(ZArchive(unshield::AsyncArchive::new(inner).await?))
    }
}

#[async_trait::async_trait(?Send)]
impl<R> super::Filesystem for ZArchive<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    type Handle = smol::io::Cursor<Vec<u8>>;
    async fn open(&mut self, path: &[&str]) -> Result<Self::Handle> {
        let arcpath = path.join("\\");
        let data = self.0.load(&arcpath).await?;
        Ok(smol::io::Cursor::new(data))
    }
}
