use anyhow::Result;
use smol::io::AsyncWriteExt;

#[derive(Debug)]
pub struct LocalFilesystem {
    pub root: std::path::PathBuf,
}

impl LocalFilesystem {
    pub fn new<P>(root: P) -> Self where P: AsRef<std::path::Path> {
        LocalFilesystem {
            root: root.as_ref().to_owned(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl super::Filesystem for LocalFilesystem {
    type Handle = smol::Unblock<std::fs::File>;
    async fn open(&mut self, path: &[&str]) -> Result<Self::Handle> {
        let mut subpath = self.root.clone();
        for part in path {
            subpath.push(part);
        }
        let file = std::fs::File::open(subpath)?;
        Ok(smol::Unblock::new(file))
    }
}

#[async_trait::async_trait(?Send)]
impl super::FilesystemWrite for LocalFilesystem {
    async fn write(&mut self, path: &[&str], data: &[u8]) -> Result<()> {
        let mut subpath = self.root.clone();
        for part in path {
            subpath.push(part);
        }
        if let Some(ref parent) = subpath.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(subpath)?;
        Ok(smol::Unblock::new(file).write_all(data).await?)
    }
}
