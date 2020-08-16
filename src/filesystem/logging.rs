use super::{Filesystem, FilesystemWrite};
use anyhow::Result;

#[derive(Debug)]
pub struct LoggingFilesystem<T> {
    inner: T,
    name: String,
}

impl<T> LoggingFilesystem<T> {
    pub fn new<S>(name: S, inner: T) -> Self
    where
        S: AsRef<str>,
    {
        LoggingFilesystem {
            inner,
            name: name.as_ref().to_owned(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<T> Filesystem for LoggingFilesystem<T>
where
    T: Filesystem,
{
    type Handle = T::Handle;

    async fn open(&self, path: &[&str]) -> Result<Self::Handle> {
        let nicepath = format!("[{}]/{}", self.name, path.join("/"));
        println!("opening {}", nicepath);
        self.inner.open(path).await
    }
}

#[async_trait::async_trait(?Send)]
impl<T> FilesystemWrite for LoggingFilesystem<T>
where
    T: FilesystemWrite,
{
    async fn write(&mut self, path: &[&str], data: &[u8]) -> Result<()> {
        let nicepath = format!("[{}]/{}", self.name, path.join("/"));
        println!("writing {}", nicepath);
        self.inner.write(path, data).await
    }
}
