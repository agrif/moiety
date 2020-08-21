use super::{MhkArchive, MhkError, MhkFormat, Narrow};
use crate::filesystem::Filesystem;

use anyhow::Result;
use smol::io::{AsyncSeek, BufReader};

use crate::{ResourceMap, ResourceMapList, Stack};
use std::collections::HashMap;

pub struct MhkMap<F, S>
where
    F: Filesystem,
{
    filesystem: F,
    stackfiles: HashMap<S, Vec<String>>,
    stacks: HashMap<S, Vec<MhkArchive<F::Handle>>>,
}

impl<F, S> MhkMap<F, S>
where
    F: Filesystem,
    F::Handle: AsyncSeek,
    S: Stack + Copy,
{
    pub fn new(filesystem: F, stackfiles: HashMap<S, Vec<&str>>) -> Self {
        MhkMap {
            filesystem,
            stackfiles: stackfiles
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().map(|s| (*s).to_owned()).collect()))
                .collect(),
            stacks: HashMap::with_capacity(S::all().len()),
        }
    }

    fn stack_file_names(&self, stack: S) -> Vec<String> {
        if let Some(names) = self.stackfiles.get(&stack) {
            names.clone()
        } else {
            vec![format!("{}.MHK", stack.name())]
        }
    }

    async fn ensure_stack(&mut self, stack: S) -> Result<()> {
        // make sure this stack is loaded
        if !self.stacks.contains_key(&stack) {
            let names = self.stack_file_names(stack);
            let mut archives = Vec::with_capacity(names.len());
            for n in names.iter() {
                let path = &[n.as_ref()];
                let handle = self.filesystem.open(path).await?;
                archives.push(MhkArchive::new(handle).await?);
            }
            self.stacks.insert(stack, archives);
        }
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<F, S> ResourceMap for MhkMap<F, S>
where
    F: Filesystem,
    F::Handle: AsyncSeek,
    S: Stack + Copy,
{
    type Handle = Narrow<BufReader<F::Handle>>;
    type Stack = S;
    type Format = MhkFormat;

    fn format(&self) -> &Self::Format {
        &MhkFormat
    }

    async fn open_raw(
        &mut self,
        stack: Self::Stack,
        typ: &str,
        id: u16,
        _ext: &str,
    ) -> Result<Self::Handle> {
        self.ensure_stack(stack).await?;
        for arc in self.stacks.get(&stack).unwrap() {
            let rsrc = arc.open(typ, id);
            match rsrc {
                Ok(r) => return Ok(r),
                Err(_) => (), // we should only ignore if ResourceNotFound, but
            }
        }

        anyhow::bail!(MhkError::ResourceNotFound(
            Some(stack.name().to_owned()),
            typ.to_owned(),
            id,
        ));
    }
}

#[async_trait::async_trait(?Send)]
impl<F, S> ResourceMapList for MhkMap<F, S>
where
    F: Filesystem,
    F::Handle: AsyncSeek,
    S: Stack + Copy,
{
    async fn list(&mut self, stack: <Self as ResourceMap>::Stack, typ: &str) -> Result<Vec<u16>> {
        self.ensure_stack(stack).await?;
        let mut ret = vec![];
        for arc in self.stacks.get(&stack).unwrap() {
            if let Some(rs) = arc.resources.get(typ) {
                for (id, _) in rs {
                    ret.push(*id);
                }
            }
        }
        ret.sort();
        Ok(ret)
    }
}
