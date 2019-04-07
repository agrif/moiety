use crate::{
    ConvertError,
    Format,
    FormatFor,
    FormatWriteFor,
    ResourceMap,
    ResourceMapList,
    ResourceMapWrite,
    ResourceType,
    Stack,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Fail)]
pub enum ResourcesError<M: failure::Fail, F: failure::Fail> {
    #[fail(display = "map error: {}", _0)]
    Map(#[cause] M),
    #[fail(display = "format error: {}", _0)]
    Format(#[cause] F),
}

pub struct Resources<M, F> {
    map: M,
    format: F,
}

impl<M, F> Resources<M, F>
where
    M: ResourceMap,
    F: Format<M::Handle>,
{
    pub fn new(map: M, format: F) -> Self { Resources { map, format } }

    pub async fn open_raw<'a, R: ResourceType + 'a>(
        &'a self,
        stack: M::Stack,
        typ: R,
        id: u16,
    ) -> Result<M::Handle, ResourcesError<M::Error, F::Error>>
    where
        F: FormatFor<M::Handle, R>,
    {
        let handle = await!(self.map.open_raw(&self.format, stack, typ, id))
            .map_err(ResourcesError::Map)?;
        Ok(handle)
    }

    pub async fn open<'a, R: ResourceType + 'a>(
        &'a self,
        stack: M::Stack,
        typ: R,
        id: u16,
    ) -> Result<R::Data, ResourcesError<M::Error, F::Error>>
    where
        F: FormatFor<M::Handle, R>,
    {
        let handle = await!(self.map.open_raw(&self.format, stack, typ, id))
            .map_err(ResourcesError::Map)?;
        let res = await!(self.format.convert(handle))
            .map_err(ResourcesError::Format)?;
        Ok(res)
    }

    pub async fn write_resource_to<'a, R: ResourceType + 'a, Mw: 'a, Fw: 'a>(
        &'a self,
        other: &'a mut Resources<Mw, Fw>,
        stack: M::Stack,
        typ: R,
        id: u16,
    ) -> Result<
        (),
        ConvertError<
            ResourcesError<M::Error, F::Error>,
            ResourcesError<Mw::Error, Fw::WriteError>,
        >,
    >
    where
        F: FormatFor<M::Handle, R>,
        Mw: ResourceMapWrite<Stack = M::Stack>,
        Fw: FormatFor<Mw::Handle, R> + FormatWriteFor<M::Handle, R, F>,
        M::Stack: Clone,
    {
        let handle =
            await!(self.map.open_raw(&self.format, stack.clone(), typ, id))
                .map_err(|e| ConvertError::Read(ResourcesError::Map(e)))?;
        let data =
            await!(other.format.write(handle, &self.format)).map_err(|e| {
                match e {
                    ConvertError::Read(e) => {
                        ConvertError::Read(ResourcesError::Format(e))
                    },
                    ConvertError::Write(e) => {
                        ConvertError::Write(ResourcesError::Format(e))
                    },
                }
            })?;
        await!(other.map.write_raw(&other.format, stack, typ, id, &data))
            .map_err(|e| ConvertError::Write(ResourcesError::Map(e)))
    }

    pub async fn write_stack_to<'a, R: ResourceType + 'a, Mw: 'a, Fw: 'a>(
        &'a self,
        other: &'a mut Resources<Mw, Fw>,
        stack: M::Stack,
        typ: R,
    ) -> Result<
        (),
        ConvertError<
            ResourcesError<M::Error, F::Error>,
            ResourcesError<Mw::Error, Fw::WriteError>,
        >,
    >
    where
        M: ResourceMapList,
        F: FormatFor<M::Handle, R>,
        Mw: ResourceMapWrite<Stack = M::Stack>,
        Fw: FormatFor<Mw::Handle, R> + FormatWriteFor<M::Handle, R, F>,
        M::Stack: Clone,
    {
        for id in await!(self.map.list(stack.clone(), typ))
            .map_err(|e| ConvertError::Read(ResourcesError::Map(e)))?
        {
            await!(self.write_resource_to(other, stack.clone(), typ, id))?;
        }
        Ok(())
    }

    pub async fn write_to<'a, R: ResourceType + 'a, Mw: 'a, Fw: 'a>(
        &'a self,
        other: &'a mut Resources<Mw, Fw>,
        typ: R,
    ) -> Result<
        (),
        ConvertError<
            ResourcesError<M::Error, F::Error>,
            ResourcesError<Mw::Error, Fw::WriteError>,
        >,
    >
    where
        M: ResourceMapList,
        F: FormatFor<M::Handle, R>,
        Mw: ResourceMapWrite<Stack = M::Stack>,
        Fw: FormatFor<Mw::Handle, R> + FormatWriteFor<M::Handle, R, F>,
        M::Stack: Stack,
    {
        for stack in M::Stack::all() {
            await!(self.write_stack_to(other, stack.clone(), typ))?;
        }
        Ok(())
    }
}
