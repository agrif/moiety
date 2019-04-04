use crate::filesystem::{Filesystem, FilesystemWrite};
use crate::{Stack, ResourceMap, ResourceMapWrite, ResourceType, FormatFor};
use crate::future::*;

#[derive(Debug)]
pub struct DirectMap<F, S> {
    filesystem: F,
    stack: std::marker::PhantomData<S>,
}

impl<F, S> DirectMap<F, S> {
    pub fn new(filesystem: F) -> Self {
        DirectMap {
            filesystem,
            stack: std::marker::PhantomData,
        }
    }
}

impl<F, S> ResourceMap for DirectMap<F, S> where F: Filesystem, S: Stack {
    type Handle = F::Handle;
    type Error = std::io::Error;
    type Stack = S;
    fn open_raw<'a, T: ResourceType + 'a, Fmt: FormatFor<Self::Handle, T>>(&'a self, fmt: &'a Fmt, stack: Self::Stack, typ: T, id: u16) -> Fut<'a, Result<Self::Handle, Self::Error>> {
        fut!({
            let fname = [
                stack.name(),
                typ.name(),
                &format!("{:05}{}", id, fmt.extension().unwrap_or(""))
            ];
            await!(self.filesystem.open(&fname))
        })
    }
}

impl<F, S> ResourceMapWrite for DirectMap<F, S> where F: FilesystemWrite, S: Stack {
    fn write_raw<'a, T: ResourceType + 'a, Fmt: FormatFor<Self::Handle, T>>(&'a mut self, fmt: &'a Fmt, stack: Self::Stack, typ: T, id: u16, data: &'a [u8]) -> Fut<'a, Result<(), Self::Error>> {
        fut!({
            let fname = [
                stack.name(),
                typ.name(),
                &format!("{:05}{}", id, fmt.extension().unwrap_or(""))
            ];
            await!(self.filesystem.write(&fname, data))
        })
    }
}
