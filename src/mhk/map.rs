use std::collections::HashMap;
use super::{MhkArchive, MhkError};
use crate::filesystem::{Filesystem, Buffered, Narrow};
use crate::{FormatFor, Stack, ResourceMap, ResourceMapList, ResourceType};
use crate::future::*;

pub struct MhkMap<F, S> where F: Filesystem {
    filesystem: F,
    stackfiles: HashMap<S, Vec<String>>,
    stacks: futures::lock::Mutex<HashMap<S, Vec<MhkArchive<F::Handle>>>>,
}

impl<F, S> MhkMap<F, S> where F: Filesystem, S: Stack {
    pub fn new(filesystem: F, stackfiles: HashMap<S, Vec<&str>>) -> Self {
        MhkMap {
            filesystem,
            stackfiles: stackfiles.iter().map(|(k, v)| {
                (*k, v.iter().map(|s| (*s).to_owned()).collect())
            }).collect(),
            stacks: futures::lock::Mutex::new(HashMap::with_capacity(8 /* FIXME number of stacks */)),
        }
    }

    fn stack_file_names(&self, stack: S) -> Vec<String> {
        if let Some(names) = self.stackfiles.get(&stack) {
            names.clone()
        } else {
            vec![format!("{}_Data.MHK", stack.letter())]
        }
    }

    async fn ensure_stack(&self, stack: S) -> Result<(), MhkError> {
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
        Ok(())
    }
}

impl<F, S> ResourceMap for MhkMap<F, S> where F: Filesystem, S: Stack {
    type Handle = Narrow<std::rc::Rc<Buffered<F::Handle>>>;
    type Error = MhkError;
    type Stack = S;
    fn open_raw<'a, T: ResourceType + 'a, Fmt: FormatFor<Self::Handle, T>>(&'a self, _fmt: &'a Fmt, stack: S, typ: T, id: u16) -> Fut<'a, Result<Self::Handle, Self::Error>> {
        fut!({
            await!(self.ensure_stack(stack))?;
            let stacks = await!(self.stacks.lock());
            for arc in stacks.get(&stack).unwrap() {
                let rsrc = arc.open(typ, id);
                if rsrc.is_ok() {
                    return rsrc;
                }
            }
            
            Err(MhkError::ResourceNotFound(Some(stack.name()), typ.name(), id))
        })
    }
}

impl<F, S> ResourceMapList for MhkMap<F, S> where F: Filesystem, S: Stack {
    fn list<'a, T: ResourceType + 'a>(&'a self, stack: Self::Stack, typ: T) -> Fut<'a, Result<Vec<u16>, Self::Error>> {
        fut!({
            await!(self.ensure_stack(stack))?;
            let stacks = await!(self.stacks.lock());
            let mut ret = vec![];
            for arc in stacks.get(&stack).unwrap() {
                if let Some(rs) = arc.resources.get(typ.name()) {
                    for (id, _) in rs {
                        ret.push(*id);
                    }
                }
            }
            ret.sort();
            Ok(ret)
        })
    }
}
