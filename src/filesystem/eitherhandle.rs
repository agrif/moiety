use smol::io::{AsyncRead, AsyncSeek, Error, SeekFrom};

use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct EitherHandle<A, B>(either::Either<A, B>);

impl<A, B> EitherHandle<A, B> {
    pub(crate) fn left(a: A) -> Self {
        EitherHandle(either::Left(a))
    }

    pub(crate) fn right(b: B) -> Self {
        EitherHandle(either::Right(b))
    }
}

impl<A, B> AsyncRead for EitherHandle<A, B>
where
    A: AsyncRead + Unpin,
    B: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        match *self {
            EitherHandle(either::Left(ref mut ha)) => Pin::new(ha).poll_read(cx, buf),
            EitherHandle(either::Right(ref mut hb)) => Pin::new(hb).poll_read(cx, buf),
        }
    }
}

impl<A, B> AsyncSeek for EitherHandle<A, B>
where
    A: AsyncSeek + Unpin,
    B: AsyncSeek + Unpin,
{
    fn poll_seek(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        pos: SeekFrom,
    ) -> Poll<Result<u64, Error>> {
        match *self {
            EitherHandle(either::Left(ref mut ha)) => Pin::new(ha).poll_seek(cx, pos),
            EitherHandle(either::Right(ref mut hb)) => Pin::new(hb).poll_seek(cx, pos),
        }
    }
}
