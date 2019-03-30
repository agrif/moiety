use std::io::Seek;
use std::io::Read;
use crate::future::*;

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

impl super::Filesystem for LocalFilesystem {
    type Handle = LocalHandle;
    fn open<'a>(&'a self, path: &'a [&str]) ->  FutureObjIO<'a, Self::Handle> {
        Box::pin(async move {
            let mut subpath = self.root.clone();
            for part in path {
                subpath.push(part);
            }
            let file = std::fs::File::open(subpath);
            file.map(|f| LocalHandle {
                file: futures::lock::Mutex::new(f),
            })
        })
    }
}

#[derive(Debug)]
pub struct LocalHandle {
    pub file: futures::lock::Mutex<std::fs::File>,
}

impl super::AsyncRead for LocalHandle {
    fn read_at<'a>(&'a self, pos: u64, buf: &'a mut [u8]) -> FutureObjIO<'a, usize> {
        Box::pin(async move {
            let mut file = await!(self.file.lock());
            file.seek(std::io::SeekFrom::Start(pos))
                .and_then(|_| file.read(buf))
        })
    }

    fn read_exact_at<'a>(&'a self, pos: u64, buf: &'a mut [u8]) -> FutureObjIO<'a, ()> {
        Box::pin(async move {
            let mut file = await!(self.file.lock());
            file.seek(std::io::SeekFrom::Start(pos))
                .and_then(|_| file.read_exact(buf))
        })
    }
}
