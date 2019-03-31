use crate::{Stack, Filesystem, ResourceMap, ResourceType};
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
    fn open_raw<'a, T: ResourceType + 'a>(&'a self, stack: Self::Stack, typ: T, id: u16) -> FutureObjResult<'a, Self::Handle, Self::Error> {
        Box::pin((async move || {
            let fname = [
                stack.name(),
                typ.name(),
                &format!("{:05}", id)
            ];
            await!(self.filesystem.open(&fname))
        })())
    }
}
