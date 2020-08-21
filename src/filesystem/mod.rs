use anyhow::Result;
use smol::io::{AsyncRead, AsyncSeek};

mod local;
pub use local::*;

mod zarchive;
pub use zarchive::*;

mod logging;
pub use logging::*;

mod eitherhandle;
pub use eitherhandle::*;

mod product;
pub use product::*;

mod sum;
pub use sum::*;

#[async_trait::async_trait(?Send)]
pub trait Filesystem {
    type Handle: AsyncRead + AsyncSeek + Unpin;
    async fn open(&mut self, path: &[&str]) -> Result<Self::Handle>;
}

#[async_trait::async_trait(?Send)]
pub trait FilesystemWrite: Filesystem {
    async fn write(&mut self, path: &[&str], data: &[u8]) -> Result<()>;
}
