use std::collections::HashMap;
use super::{MhkArchive, MhkError};
use crate::{Stack, Filesystem, Buffered, ResourceMap, ResourceType, Narrow};
use crate::future::*;

#[derive(Debug)]
pub struct MhkMap<F, S> where F: Filesystem {
    filesystem: F,
    stacks: futures::lock::Mutex<HashMap<S, Vec<MhkArchive<F::Handle>>>>,
}

impl<F, S> MhkMap<F, S> where F: Filesystem, S: Stack {
    pub fn new(filesystem: F) -> Self {
        MhkMap {
            filesystem,
            stacks: futures::lock::Mutex::new(HashMap::with_capacity(8 /* FIXME number of stacks */)),
        }
    }

    fn stack_file_names(&self, stack: S) -> Vec<String> {
        // FIXME multiple files
        vec![format!("{}_Data.MHK", stack.letter())]
    }
}

impl<F, S> ResourceMap for MhkMap<F, S> where F: Filesystem, S: Stack {
    type Handle = Narrow<std::rc::Rc<Buffered<F::Handle>>>;
    type Error = MhkError;
    type Stack = S;
    fn open_raw<'a, T: ResourceType + 'a>(&'a self, stack: Self::Stack, typ: T, id: u16) -> FutureObjResult<'a, Self::Handle, Self::Error> {
        Box::pin((async move || {
            let mut stacks = await!(self.stacks.lock());
            // make sure this stack is loaded
            if !stacks.contains_key(&stack) {
                let names = self.stack_file_names(stack);
                let archive_futures = names.iter().map(async move |n| {
                    let path = &[n.as_ref()];
                    let handle = await!(self.filesystem.open(path))?;
                    await!(MhkArchive::new(handle))
                });
                let archives: Result<Vec<_>, MhkError> = await!(futures::future::join_all(archive_futures)).into_iter().collect();
                stacks.insert(stack, archives?);
            }
            
            for arc in stacks.get(&stack).unwrap() {
                let rsrc = arc.open(typ, id);
                if rsrc.is_ok() {
                    return rsrc;
                }
            }
            
            Err(MhkError::ResourceNotFound(Some(stack.name()), typ.name(), id))
        })())
    }
}
