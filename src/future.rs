pub use std::future::Future;
use std::pin::Pin;

// this file is a shim. by rights, everywhere these are used should be
// replaced by true async trait methods. But as of today, 03/18/2019,
// async trait methods do not work. So instead we use boxed futures.

pub type FutureObj<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type FutureObjResult<'a, T, E> = FutureObj<'a, Result<T, E>>;
pub type FutureObjIO<'a, T> = FutureObj<'a, std::io::Result<T>>;
