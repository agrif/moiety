use super::{MhkArchive, MhkError, MhkFormat, Narrow};
use super::ownedpe::OwnedPe32;
use crate::filesystem::{Filesystem, EitherHandle};

use anyhow::Result;
use smol::io::{AsyncRead, AsyncReadExt, BufReader, Cursor};

use crate::{ResourceMap, ResourceMapList, Stack};
use std::collections::HashMap;

pub struct MhkMap<F, S>
where
    F: Filesystem,
{
    filesystem: F,
    stackfiles: HashMap<S, Vec<String>>,
    stacks: HashMap<S, Vec<Archive<F::Handle>>>,
}

enum Archive<H: AsyncRead> {
    Mhk(MhkArchive<H>),
    Pe32(OwnedPe32),
}

impl<F, S> MhkMap<F, S>
where
    F: Filesystem,
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
                let mut handle = self.filesystem.open(path).await?;
                if n.ends_with(".exe") {
                    let mut data = Vec::with_capacity(600000);
                    handle.read_to_end(&mut data).await?;
                    archives.push(Archive::Pe32(OwnedPe32::new(data)?));
                } else {
                    archives.push(Archive::Mhk(MhkArchive::new(handle).await?));
                }
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
    S: Stack + Copy,
{
    type Handle = EitherHandle<Narrow<BufReader<F::Handle>>, Cursor<Vec<u8>>>;
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
            let rsrc = match arc {
                Archive::Mhk(marc) => EitherHandle::left(marc.open(typ, id)),
                Archive::Pe32(parc) => EitherHandle::right(parc.open(typ, id)),
            };
            match rsrc.factor_error() {
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
    S: Stack + Copy,
{
    async fn list(&mut self, stack: <Self as ResourceMap>::Stack, typ: &str) -> Result<Vec<u16>> {
        self.ensure_stack(stack).await?;
        let mut ret = vec![];
        for arc in self.stacks.get(&stack).unwrap() {
            match arc {
                Archive::Mhk(marc) => {
                    if let Some(rs) = marc.resources.get(typ) {
                        for (id, _) in rs {
                            ret.push(*id);
                        }
                    }
                }
                Archive::Pe32(parc) => {
                    if let Some(rs) = parc.get(typ) {
                        for id in rs {
                            ret.push(id);
                        }
                    }
                }
            }
        }
        ret.sort();
        Ok(ret)
    }
}
