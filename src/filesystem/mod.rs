use crate::future::*;

mod local;
pub use local::*;

mod logging;
pub use logging::*;

mod narrow;
pub use narrow::*;

mod buffered;
pub use buffered::*;

pub trait Filesystem {
    type Handle: AsyncRead;
    fn open<'a>(&'a self, path: &'a [&str]) -> FutureObjIO<'a, Self::Handle>;
}

pub trait AsyncRead: Sized {
    fn read_at<'a>(&'a self, pos: u64, buf: &'a mut [u8]) -> FutureObjIO<'a, usize>;

    fn read_exact_at<'a>(&'a self, mut pos: u64, mut buf: &'a mut [u8]) -> FutureObjIO<'a, ()> {
        Box::pin((async move || {
            while !buf.is_empty() {
                match await!(self.read_at(pos, buf))? {
                    0 => return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "failed to fill whole buffer"
                    )),
                    n => {
                        let tmp = buf;
                        buf = &mut tmp[n..];
                        pos += n as u64;
                    },
                }
            }
            Ok(())
        })())
    }

    // size is a hint, not an absolute
    fn read_until_at<'a>(&'a self, mut pos: u64, size: u64, delim: u8, buf: &'a mut Vec<u8>) -> FutureObjIO<'a, usize> {
        Box::pin((async move || {
            let mut smallbuf = vec![0; size as usize];
            let mut read = 0;
            loop {
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
                    }
                }
            }
            Ok(read)
        })())
    }

    fn narrow(self: &std::rc::Rc<Self>, offset: u64, size: u64) -> Narrow<std::rc::Rc<Self>> {
        Narrow::new(self.clone(), offset, size)
    }

    fn buffer(self, bufsize: u64) -> Buffered<Self> {
        Buffered::new(self, bufsize)
    }
}

impl<T> AsyncRead for std::rc::Rc<T> where T: AsyncRead {
    fn read_at<'a>(&'a self, pos: u64, buf: &'a mut [u8]) -> FutureObjIO<'a, usize> {
        (**self).read_at(pos, buf)
    }

    fn read_exact_at<'a>(&'a self, pos: u64, buf: &'a mut [u8]) -> FutureObjIO<'a, ()> {
        (**self).read_exact_at(pos, buf)
    }

    fn read_until_at<'a>(&'a self, pos: u64, size: u64, delim: u8, buf: &'a mut Vec<u8>) -> FutureObjIO<'a, usize> {
        (**self).read_until_at(pos, size, delim, buf)
    }

}
