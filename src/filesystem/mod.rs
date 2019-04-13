use crate::future::*;
use std::io::Result;

mod local;
pub use local::*;

mod logging;
pub use logging::*;

mod narrow;
pub use narrow::*;

mod buffered;
pub use buffered::*;

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

// used for read_to_end
const RESERVATION_SIZE: usize = 32;

pub trait Filesystem {
    type Handle: AsyncRead;
    fn open<'a>(&'a self, path: &'a [&str]) -> Fut<'a, Result<Self::Handle>>;
}

pub trait FilesystemWrite: Filesystem {
    fn write<'a>(
        &'a mut self,
        path: &'a [&str],
        data: &'a [u8],
    ) -> Fut<'a, Result<()>>;
}

pub trait AsyncRead: Sized {
    fn read_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, Result<usize>>;

    fn read_exact_at<'a>(
        &'a self,
        mut pos: u64,
        mut buf: &'a mut [u8],
    ) -> Fut<'a, Result<()>> {
        fut!({
            while !buf.is_empty() {
                match await!(self.read_at(pos, buf))? {
                    0 => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "failed to fill whole buffer",
                        ))
                    },
                    n => {
                        let tmp = buf;
                        buf = &mut tmp[n..];
                        pos += n as u64;
                    },
                }
            }
            Ok(())
        })
    }

    fn read_until_end_at<'a>(
        &'a self,
        mut pos: u64,
        buf: &'a mut Vec<u8>,
    ) -> Fut<'a, Result<usize>> {
        fut!({
            let start_len = buf.len();
            let mut g = Guard {
                len: buf.len(),
                buf,
            };
            let ret;
            loop {
                if g.len == g.buf.len() {
                    unsafe {
                        g.buf.reserve(RESERVATION_SIZE);
                        let capacity = g.buf.capacity();
                        g.buf.set_len(capacity);
                        // FIXME initialize here...
                    }
                }

                match await!(self.read_at(pos, &mut g.buf[g.len..])) {
                    Ok(0) => {
                        ret = Ok(g.len - start_len);
                        break;
                    },
                    Ok(n) => {
                        g.len += n;
                        pos += n as u64;
                    },
                    // FIXME interrupted
                    Err(e) => {
                        ret = Err(e);
                        break;
                    },
                }
            }

            ret
        })
    }

    // size is a hint, not an absolute
    fn read_until_at<'a>(
        &'a self,
        mut pos: u64,
        delim: u8,
        buf: &'a mut Vec<u8>,
    ) -> Fut<'a, Result<usize>> {
        fut!({
            let mut smallbuf = vec![0; RESERVATION_SIZE];
            let mut read = 0;
            loop {
                // FIXME handle interrupted
                let readpart = await!(self.read_at(pos, &mut smallbuf))?;
                if readpart == 0 {
                    break;
                }
                match memchr::memchr(delim, &smallbuf[..readpart]) {
                    None => {
                        buf.extend_from_slice(&smallbuf[..readpart]);
                        read += readpart;
                        pos += readpart as u64;
                    },
                    Some(i) => {
                        buf.extend_from_slice(&smallbuf[..i + 1]);
                        read += i + 1;
                        break;
                    },
                }
            }
            Ok(read)
        })
    }

    fn narrow(
        self: &std::rc::Rc<Self>,
        offset: u64,
        size: u64,
    ) -> Narrow<std::rc::Rc<Self>> {
        Narrow::new(self.clone(), offset, size)
    }

    fn buffer(self, bufsize: u64) -> Buffered<Self> {
        Buffered::new(self, bufsize)
    }
}

impl<T> AsyncRead for std::rc::Rc<T>
where
    T: AsyncRead,
{
    fn read_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, Result<usize>> {
        (**self).read_at(pos, buf)
    }

    fn read_exact_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, Result<()>> {
        (**self).read_exact_at(pos, buf)
    }

    fn read_until_at<'a>(
        &'a self,
        pos: u64,
        delim: u8,
        buf: &'a mut Vec<u8>,
    ) -> Fut<'a, Result<usize>> {
        (**self).read_until_at(pos, delim, buf)
    }
}
