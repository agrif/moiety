use crate::shims::*;
use moiety::{
    filesystem::{
        AsyncRead,
        Filesystem,
    },
    future::*,
};
use std::io::Result;

#[derive(Debug)]
pub struct WebFilesystem {
    pub root: String,
}

impl WebFilesystem {
    pub fn new<S>(root: S) -> Self
    where
        S: AsRef<str>,
    {
        WebFilesystem {
            root: root.as_ref().to_owned(),
        }
    }
}

impl Filesystem for WebFilesystem {
    type Handle = WebHandle;

    fn open<'a>(&'a self, path: &'a [&str]) -> Fut<'a, Result<Self::Handle>> {
        fut!({
            Ok(WebHandle {
                path: format!("{}/{}", self.root, path.join("/")),
            })
        })
    }
}

pub struct WebHandle {
    pub path: String,
}

impl WebHandle {
    pub async fn make_request(
        &self,
        start: u64,
        end: Option<u64>,
    ) -> Result<web_sys::Response> {
        let window = web_sys::window().unwrap();
        let request = web_sys::Request::new_with_str(&self.path).unwrap();
        if let Some(endi) = end {
            request
                .headers()
                .set("Range", &format!("bytes={}-{}", start, endi - 1))
                .unwrap();
        } else if start > 0 {
            request
                .headers()
                .set("Range", &format!("bytes={}-", start))
                .unwrap();
        }

        await!(unpromise(window.fetch_with_request(&request)))
    }
}

impl AsyncRead for WebHandle {
    fn read_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut [u8],
    ) -> Fut<'a, Result<usize>> {
        fut!({
            let response =
                await!(self.make_request(pos, Some(pos + buf.len() as u64)))?;
            if response.status() == 416 {
                // we requested something of 0 bytes...
                return Ok(0);
            }
            let arrbuf: js_sys::ArrayBuffer =
                await!(unpromise(unerror(response.array_buffer())?))?;
            let arr = js_sys::Uint8Array::new(&arrbuf);
            let read = arr.length() as usize;
            assert!(buf.len() >= read);
            arr.copy_to(&mut buf[..read]);
            Ok(read)
        })
    }

    fn read_until_end_at<'a>(
        &'a self,
        pos: u64,
        buf: &'a mut Vec<u8>,
    ) -> Fut<'a, Result<usize>> {
        fut!({
            let response = await!(self.make_request(pos, None))?;
            let arrbuf: js_sys::ArrayBuffer =
                await!(unpromise(unerror(response.array_buffer())?))?;
            let arr = js_sys::Uint8Array::new(&arrbuf);
            let read = arr.length() as usize;
            buf.reserve(read);
            let start = buf.len();
            unsafe {
                buf.set_len(start + read);
                arr.copy_to(&mut buf[start..]);
            }
            Ok(read)
        })
    }
}
