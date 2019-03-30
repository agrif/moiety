use crate::future::*;

#[derive(Debug)]
pub struct LoggingFilesystem<T> {
    inner: T,
    name: String,
}

impl<T> LoggingFilesystem<T> {
    pub fn new<S>(name: S, inner: T) -> Self where S: AsRef<str> {
        LoggingFilesystem {
            inner,
            name: name.as_ref().to_owned(),
        }
    }
}

async fn bracket<F, Fn, E, R, U>(message: String, future: F, map: Fn) -> Result<U, E> where F: Future<Output = Result<R, E>>, Fn: FnOnce(R) -> U {
    println!("[begin] {}", message);
    let res = await!(future);
    match res {
        Ok(r) => {
            println!("[ end ] {}", message);
            Ok(map(r))
        },
        Err(e) => {
            println!("[ !!! ] {}", message);
            Err(e)
        },
    }
}

impl<T> super::Filesystem for LoggingFilesystem<T> where T: super::Filesystem {
    type Handle = LoggingHandle<T::Handle>;
    fn open<'a>(&'a self, path: &'a [&str]) -> FutureObjIO<'a, Self::Handle> {
        Box::pin(async move {
            let nicepath = format!("[{}]/{}", self.name, path.join("/"));
            let message = format!("opening {}", nicepath);
            await!(bracket(message, self.inner.open(path), |h| {
                LoggingHandle {
                    inner: h,
                    name: nicepath,
                }
            }))
        })
    }
}

#[derive(Debug)]
pub struct LoggingHandle<T> {
    inner: T,
    name: String,
}

impl<T> super::AsyncRead for LoggingHandle<T> where T: super::AsyncRead {
    fn read_at<'a>(&'a self, pos: u64, buf: &'a mut [u8]) -> FutureObjIO<'a, usize> {
        Box::pin(async move {
            let message = format!("reading {}-{} of {}", pos, pos + buf.len() as u64, self.name);
            await!(bracket(message, self.inner.read_at(pos, buf), |x| x))
        })
    }
}
