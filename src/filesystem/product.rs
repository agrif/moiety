use super::{EitherHandle, Filesystem};

use anyhow::Result;

#[async_trait::async_trait(?Send)]
impl<A, B> Filesystem for (A, B)
where
    A: Filesystem,
    B: Filesystem,
{
    type Handle = EitherHandle<A::Handle, B::Handle>;
    async fn open(&mut self, path: &[&str]) -> Result<Self::Handle> {
        match self.0.open(path).await {
            Ok(h) => Ok(EitherHandle::left(h)),
            Err(_) => Ok(EitherHandle::right(self.1.open(path).await?)),
        }
    }
}
