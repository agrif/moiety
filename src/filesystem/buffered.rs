use crate::future::*;

#[derive(Debug)]
pub struct Buffered<T> {
    bufsize: u64,
    buffer: futures::lock::Mutex<BufData>,
    inner: T,
}

struct BufData {
    buffer: Vec<u8>,
    start: u64,
}

impl<T> Buffered<T> where T: super::AsyncRead {
    pub fn new(inner: T, bufsize: u64) -> Self {
        Buffered {
            inner,
            bufsize,
            buffer: futures::lock::Mutex::new(BufData {
                buffer: Vec::with_capacity(bufsize as usize),
                start: 0,
            }),
        }
    }

    async fn fill_buffer<'a>(&'a self, pos: u64, bufdata: &'a mut BufData) -> std::io::Result<()> {
        // FIXME panics can cause the new size to never be set...
        unsafe {
            bufdata.buffer.set_len(self.bufsize as usize);
        }
        // FIXME this really ought to do something like read_exact
        let size = await!(self.inner.read_at(pos, &mut bufdata.buffer))?;
        unsafe {
            bufdata.buffer.set_len(size);
        }
        bufdata.start = pos;
        Ok(())
    }

    async fn ensure_buffer<'a>(&'a self, pos: u64) -> std::io::Result<futures::lock::MutexGuard<BufData>> {
        let mut bufdata = await!(self.buffer.lock());
        if pos < bufdata.start || pos >= bufdata.start + bufdata.buffer.len() as u64 {
            await!(self.fill_buffer(pos, &mut *bufdata))?;
        }
        Ok(bufdata)
    }
}

impl<T> super::AsyncRead for Buffered<T> where T: super::AsyncRead {
    fn read_at<'a>(&'a self, pos: u64, buf: &'a mut [u8]) -> FutureObjIO<'a, usize> {
        Box::pin((async move || {
            let bufdata = await!(self.ensure_buffer(pos))?;
            let buffer_start = pos as usize - bufdata.start as usize;
            let buffer_end = bufdata.buffer.len().min(buffer_start as usize + buf.len());
            let slice_end = buffer_end - buffer_start;
            buf[..slice_end].clone_from_slice(&bufdata.buffer[buffer_start..buffer_end]);
            Ok(slice_end)
        })())
    }
}
