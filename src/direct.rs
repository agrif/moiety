use crate::filesystem::{Filesystem, FilesystemWrite};
use crate::{ResourceMap, ResourceMapWrite, Stack};

use anyhow::Result;

#[derive(Debug)]
pub struct DirectMap<F, Fmt, S> {
    filesystem: F,
    format: Fmt,
    stack: std::marker::PhantomData<S>,
}

impl<F, Fmt, S> DirectMap<F, Fmt, S> {
    pub fn new(filesystem: F, format: Fmt) -> Self {
        DirectMap {
            filesystem,
            format,
            stack: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<F, Fmt, S> ResourceMap for DirectMap<F, Fmt, S>
where
    F: Filesystem,
    S: Stack,
{
    type Handle = F::Handle;
    type Stack = S;
    type Format = Fmt;

    fn format(&self) -> &Self::Format {
        &self.format
    }

    async fn open_raw(&self, stack: Self::Stack, typ: &str, id: u16, ext: &str)
                      -> Result<Self::Handle>
    {
        let fname = [
            stack.name(),
            typ,
            &format!("{:05}{}", id, ext),
        ];
        self.filesystem.open(&fname).await
    }
}

#[async_trait::async_trait(?Send)]
impl<F, Fmt, S> ResourceMapWrite for DirectMap<F, Fmt, S>
where
    F: FilesystemWrite,
    S: Stack,
{
    async fn write_raw(
        &mut self,
        stack: <Self as ResourceMap>::Stack,
        typ: &str,
        id: u16,
        ext: &str,
        data: &[u8],
    ) -> Result<()>
    {
        let fname = [
            stack.name(),
            typ,
            &format!("{:05}{}", id, ext),
        ];
        self.filesystem.write(&fname, data).await
    }
}
