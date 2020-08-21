use anyhow::Result;

// FIXME this should be async

#[derive(Debug)]
pub struct ZArchive<R>(unshield::Archive<R>);

impl<R> ZArchive<R> where R: std::io::Read + std::io::Seek {
    pub async fn new(inner: R) -> Result<Self> {
        Ok(ZArchive(unshield::Archive::new(inner)?))
    }
}

#[async_trait::async_trait(?Send)]
impl<R> super::Filesystem for ZArchive<R> where R: std::io::Read + std::io::Seek {
    type Handle = smol::io::Cursor<Vec<u8>>;
    async fn open(&mut self, path: &[&str]) -> Result<Self::Handle> {
        let arcpath = path.join("\\");
        let data = self.0.load(&arcpath)?;
        Ok(smol::io::Cursor::new(data))
    }
}
