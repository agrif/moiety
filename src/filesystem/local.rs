use crate::future::*;
use std::io::{
    Read,
    Result,
    Seek,
    Write,
};

#[derive(Debug)]
pub struct LocalFilesystem {
    pub root: std::path::PathBuf,
}

impl LocalFilesystem {
    pub fn new<P>(root: P) -> Self
    where
        P: AsRef<std::path::Path>,
    {
        LocalFilesystem {
            root: root.as_ref().to_owned(),
        }
    }
}

impl super::Filesystem for LocalFilesystem {
    type Handle = LocalHandle;

    fn open<'a>(&'a self, path: &'a [&str]) -> Fut<'a, Result<Self::Handle>> {
        fut!({
            let mut subpath = self.root.clone();
            for part in path {
                subpath.push(part);
            }
            let file = std::fs::File::open(subpath);
            Ok(LocalHandle {
                file: futures::lock::Mutex::new(file?),
            })
        })
    }
}

impl super::FilesystemWrite for LocalFilesystem {
    fn write<'a>(
        &'a mut self,
        path: &'a [&str],
        data: &'a [u8],
    ) -> Fut<'a, Result<()>> {
        fut!({
            let mut subpath = self.root.clone();
            for part in path {
                subpath.push(part);
            }

            if let Some(ref parent) = subpath.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = std::fs::File::create(subpath)?;
            file.write_all(data)
        })
    }
}

#[derive(Debug)]
pub struct LocalHandle {
    pub file: futures::lock::Mutex<std::fs::File>,
}

impl super::AsyncRead for LocalHandle {
    fn read_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, Result<usize>> {
        fut!({
            let mut file = await!(self.file.lock());
            file.seek(std::io::SeekFrom::Start(pos))?;
            file.read(buf)
        })
    }

    fn read_exact_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, Result<()>> {
        fut!({
            let mut file = await!(self.file.lock());
            file.seek(std::io::SeekFrom::Start(pos))?;
            file.read_exact(buf)
        })
    }
}
