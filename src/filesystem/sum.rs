use super::{EitherHandle, Filesystem};

use anyhow::Result;

#[async_trait::async_trait(?Send)]
impl<A, B> Filesystem for either::Either<A, B>
where
    A: Filesystem,
    B: Filesystem,
{
    type Handle = EitherHandle<A::Handle, B::Handle>;
    async fn open(&mut self, path: &[&str]) -> Result<Self::Handle> {
        match self {
            either::Left(a) => Ok(EitherHandle::left(a.open(path).await?)),
            either::Right(b) => Ok(EitherHandle::right(b.open(path).await?)),
        }
    }
}
