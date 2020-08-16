use anyhow::Result;
use smol::io::AsyncRead;

mod local;
pub use local::*;

mod logging;
pub use logging::*;

#[async_trait::async_trait(?Send)]
pub trait Filesystem {
    type Handle: AsyncRead + Unpin;
    async fn open(&self, path: &[&str]) -> Result<Self::Handle>;
}

#[async_trait::async_trait(?Send)]
pub trait FilesystemWrite: Filesystem {
    async fn write(&mut self, path: &[&str], data: &[u8]) -> Result<()>;
}
