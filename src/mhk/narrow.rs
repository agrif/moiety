use std::rc::Rc;
use std::cell::RefCell;
use std::io::Result;
use std::task::{Context, Poll};
use std::pin::Pin;

use smol::io::{AsyncRead, AsyncSeek, SeekFrom};

#[derive(Debug)]
pub struct Narrow<T> {
    inner: Rc<RefCell<(u64, T)>>,
    offset: u64,
    size: u64,
    pos: u64, // within narrowed region
}

impl<T> Narrow<T> {
    pub fn new(inner: Rc<RefCell<(u64, T)>>, offset: u64, size: u64) -> Self {
        Narrow {
            inner,
            offset,
            size,
            pos: 0,
        }
    }
}

impl<T> AsyncRead for Narrow<T> where T: AsyncRead + AsyncSeek + Unpin {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        mut buf: &mut [u8]
    ) -> Poll<Result<usize>>
    {
        if self.pos >= self.size {
            return Poll::Ready(Ok(0));
        }

        let mut h = self.inner.borrow_mut();
        if buf.len() as u64 > self.size - self.pos {
            buf = &mut buf[..(self.size - self.pos) as usize]
        }

        let truepos = self.offset + self.pos;
        if truepos != h.0 {
            let p = Pin::new(&mut h.1).poll_seek(cx, SeekFrom::Start(truepos));
            if let Poll::Ready(Err(e)) = p {
                return Poll::Ready(Err(e));
            } else if let Poll::Pending = p {
                return Poll::Pending;
            }
            h.0 = truepos;
        }

        let p = Pin::new(&mut h.1).poll_read(cx, buf);
        if let Poll::Ready(Ok(amt)) = p {
            h.0 += amt as u64;
            std::mem::drop(h);
            self.pos += amt as u64;
        }
        p
    }
}

impl<T> AsyncSeek for Narrow<T> where T: AsyncSeek + Unpin {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        pos: SeekFrom,
    ) -> Poll<Result<u64>>
    {
        let newpos = match pos {
            SeekFrom::Start(p) => p.min(self.size),
            SeekFrom::End(p) =>
                ((self.size as i64 + p.max(-(self.size as i64))) as u64)
                .min(self.size),
            SeekFrom::Current(p) =>
                ((self.pos as i64 + p.max(-(self.pos as i64))) as u64)
                .min(self.size),
        };
        if self.pos == newpos {
            return Poll::Ready(Ok(self.pos));
        }

        let mut h = self.inner.borrow_mut();
        let truepos = self.offset + newpos;
        if truepos == h.0 {
            return Poll::Ready(Ok(self.pos));
        }

        let p = Pin::new(&mut h.1).poll_seek(cx, SeekFrom::Start(truepos));
        if let Poll::Ready(Ok(newtruepos)) = p {
            h.0 = newtruepos;
            std::mem::drop(h);
            self.pos = newpos;
            return Poll::Ready(Ok(newpos));
        }
        p
    }
}
