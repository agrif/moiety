use crate::{
    Format, FormatWrite,
    ResourceMap, ResourceMapList, ResourceMapWrite,
    ResourceType, Stack,
};

use anyhow::Result;

pub struct Resources<M> {
    map: M,
}

impl<M> Resources<M> where M: ResourceMap {
    pub fn new(map: M) -> Self {
        Resources {
            map,
        }
    }

    pub async fn open_raw<R>(&self, stack: M::Stack, typ: R, id: u16)
                             -> Result<M::Handle>
    where
        R: ResourceType,
        M::Format: Format<R, M::Handle, R::Data>,
    {
        let fmt = self.map.format();
        let extension = fmt.extension(&typ).unwrap_or("");
        let handle = self.map.open_raw(stack, typ.name(), id, extension).await?;
        Ok(handle)
    }

    pub async fn open<R>(&self, stack: M::Stack, typ: R, id: u16)
                         -> Result<R::Data>
    where
        R: ResourceType,
        M::Format: Format<R, M::Handle, R::Data>,
    {
        let mut handle = self.open_raw(stack, typ, id).await?;
        let res = self.map.format().parse(&typ, &mut handle).await?;
        Ok(res)
    }

    pub async fn write_resource_to<R, Mw>(
        &self,
        other: &mut Resources<Mw>,
        stack: M::Stack,
        typ: R,
        id: u16
    ) -> Result<()>
    where
        R: ResourceType,
        Mw: ResourceMapWrite<Stack=M::Stack>,
        Mw::Format: FormatWrite<M::Format, R, M::Handle, R::Data>,
        M::Format: Format<R, M::Handle, R::Data>,
        M::Stack: Clone,
    {
        let mut handle = self.open_raw(stack.clone(), typ, id).await?;
        let data = other.map.format().convert(
            self.map.format(), &typ, &mut handle).await?;
        let extension =other.map.format().extension(&typ)
            .unwrap_or("").to_owned();
        other.map.write_raw(
            stack.clone(), typ.name(), id, &extension, &data).await?;
        Ok(())
    }

    pub async fn write_stack_to<R, Mw>(
        &self,
        other: &mut Resources<Mw>,
        stack: M::Stack,
        typ: R,
    ) -> Result<()>
    where
        R: ResourceType,
        M: ResourceMapList,
        Mw: ResourceMapWrite<Stack=M::Stack>,
        Mw::Format: FormatWrite<M::Format, R, M::Handle, R::Data>,
        M::Format: Format<R, M::Handle, R::Data>,
        M::Stack: Clone,
    {
        for id in self.map.list(stack.clone(), typ.name()).await? {
            self.write_resource_to(other, stack.clone(), typ, id).await?;
        }
        Ok(())
    }

    pub async fn write_to<R, Mw>(
        &self,
        other: &mut Resources<Mw>,
        typ: R,
    ) -> Result<()>
    where
        R: ResourceType,
        M: ResourceMapList,
        Mw: ResourceMapWrite<Stack=M::Stack>,
        Mw::Format: FormatWrite<M::Format, R, M::Handle, R::Data>,
        M::Format: Format<R, M::Handle, R::Data>,
        M::Stack: Clone,
    {
        for stack in M::Stack::all() {
            self.write_stack_to(other, stack.clone(), typ).await?;
        }
        Ok(())
    }
}
