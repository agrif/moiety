use crate::future::*;
use std::io::Result;

#[derive(Debug)]
pub struct Narrow<T> {
    inner: T,
    offset: u64,
    size: u64,
}

impl<T> Narrow<T> {
    pub fn new(inner: T, offset: u64, size: u64) -> Self {
        Narrow {
            inner,
            offset,
            size,
        }
    }
}

impl<T> super::AsyncRead for Narrow<T>
where
    T: super::AsyncRead,
{
    fn read_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, Result<usize>> {
        let mut len = 0;
        if pos < self.size {
            len = buf.len().min((self.size - pos) as usize);
        }
        self.inner.read_at(self.offset + pos, &mut buf[..len])
    }
}
