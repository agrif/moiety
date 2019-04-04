pub use std::future::Future;
use std::pin::Pin;

// this file is a shim. by rights, everywhere these are used should be
// replaced by true async trait methods. But as of today, 03/18/2019,
// async trait methods do not work. So instead we use boxed futures.

#[must_use]
pub struct Fut<'a, T>(pub Pin<Box<dyn Future<Output = T> + 'a>>);

impl<'a, T> std::ops::Deref for Fut<'a, T> {
    type Target = Pin<Box<dyn Future<Output = T> + 'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> std::ops::DerefMut for Fut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> Future for Fut<'a, T> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, lw: &std::task::Waker) -> std::task::Poll<Self::Output> {
        (**self).as_mut().poll(lw)
    }
}

#[macro_export]
macro_rules! fut {
    ($b:block) => {
        Fut(Box::pin((async move || $b)()))
    }
}
