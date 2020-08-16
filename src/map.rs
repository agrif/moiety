use crate::Stack;

use anyhow::Result;
use smol::io::AsyncRead;

#[async_trait::async_trait(?Send)]
pub trait ResourceMap {
    type Handle: AsyncRead + Unpin;
    type Stack: Stack;
    type Format;
    fn format(&self) -> &Self::Format;
    async fn open_raw(&self, stack: Self::Stack, typ: &str, id: u16, ext: &str)
                      -> Result<Self::Handle>;
}

#[async_trait::async_trait(?Send)]
pub trait ResourceMapList: ResourceMap {
    async fn list(&self, stack: Self::Stack, typ: &str) -> Result<Vec<u16>>;
}

#[async_trait::async_trait(?Send)]
pub trait ResourceMapWrite: ResourceMap {
    async fn write_raw(
        &mut self,
        stack: Self::Stack,
        typ: &str,
        id: u16,
        ext: &str,
        data: &[u8],
    ) -> Result<()>;
}
