use crate::{ResourceMap, ResourceMapWrite, ResourceMapList, Format, FormatFor, FormatWriteFor, ResourceType, ConvertError, Stack};

pub struct Resources<M, F, E> {
    map: M,
    format: F,
    error_phantom: std::marker::PhantomData<E>,
}

impl<M, F> Resources<M, F, M::Error> where M: ResourceMap, F: Format<M::Handle>, M::Error: From<F::Error> {
    pub fn new_with_map_error(map: M, format: F) -> Self {
        Self::new(map, format)
    }
}

impl<M, F> Resources<M, F, F::Error> where M: ResourceMap, F: Format<M::Handle>, F::Error: From<M::Error> {
    pub fn new_with_format_error(map: M, format: F) -> Self {
        Self::new(map, format)
    }
}

impl<M, F, E> Resources<M, F, E> where M: ResourceMap, F: Format<M::Handle>, E: From<M::Error> + From<F::Error> {
    pub fn new(map: M, format: F) -> Self {
        Resources {
            map,
            format,
            error_phantom: std::marker::PhantomData,
        }
    }

    pub async fn open_raw<'a, R: ResourceType + 'a>(&'a self, stack: M::Stack, typ: R, id: u16) -> Result<M::Handle, E> where F: FormatFor<M::Handle, R> {
        let handle = await!(self.map.open_raw(&self.format, stack, typ, id))?;
        Ok(handle)
    }

    pub async fn open<'a, R: ResourceType + 'a>(&'a self, stack: M::Stack, typ: R, id: u16) -> Result<R::Data, E> where F: FormatFor<M::Handle, R> {
        let handle = await!(self.map.open_raw(&self.format, stack, typ, id))?;
        let res = await!(self.format.convert(handle))?;
        Ok(res)
    }

    pub async fn write_resource_to<'a, R: ResourceType + 'a, Mw: 'a, Fw: 'a, Ew: 'a>(&'a self, other: &'a mut Resources<Mw, Fw, Ew>, stack: M::Stack, typ: R, id: u16) -> Result<(), ConvertError<E, Ew>> where F: FormatFor<M::Handle, R>, Mw: ResourceMapWrite<Stack=M::Stack>, Fw: FormatFor<Mw::Handle, R> + FormatWriteFor<M::Handle, R, F>, Ew: From<Mw::Error> + From<Fw::WriteError>, E: failure::Fail, Ew: failure::Fail, M::Stack: Clone {
        let handle = await!(self.map.open_raw(&self.format, stack.clone(), typ, id)).map_err(|e| ConvertError::Read(e.into()))?;
        let data = await!(other.format.write(handle, &self.format)).map_err(|e| {
            match e {
                ConvertError::Read(e) => ConvertError::Read(e.into()),
                ConvertError::Write(e) => ConvertError::Write(e.into()),
            }
        })?;
        await!(other.map.write_raw(&other.format, stack, typ, id, &data)).map_err(|e| ConvertError::Write(e.into()))
    }

    pub async fn write_stack_to<'a, R: ResourceType + 'a, Mw: 'a, Fw: 'a, Ew: 'a>(&'a self, other: &'a mut Resources<Mw, Fw, Ew>, stack: M::Stack, typ: R) -> Result<(), ConvertError<E, Ew>> where M: ResourceMapList, F: FormatFor<M::Handle, R>, Mw: ResourceMapWrite<Stack=M::Stack>, Fw: FormatFor<Mw::Handle, R> + FormatWriteFor<M::Handle, R, F>, Ew: From<Mw::Error> + From<Fw::WriteError>, E: failure::Fail, Ew: failure::Fail, M::Stack: Clone {
        for id in await!(self.map.list(stack.clone(), typ)).map_err(|e| ConvertError::Read(e.into()))? {
            await!(self.write_resource_to(other, stack.clone(), typ, id))?;
        }
        Ok(())
    }

    pub async fn write_to<'a, R: ResourceType + 'a, Mw: 'a, Fw: 'a, Ew: 'a>(&'a self, other: &'a mut Resources<Mw, Fw, Ew>, typ: R) -> Result<(), ConvertError<E, Ew>> where M: ResourceMapList, F: FormatFor<M::Handle, R>, Mw: ResourceMapWrite<Stack=M::Stack>, Fw: FormatFor<Mw::Handle, R> + FormatWriteFor<M::Handle, R, F>, Ew: From<Mw::Error> + From<Fw::WriteError>, E: failure::Fail, Ew: failure::Fail, M::Stack: Stack {
        for stack in M::Stack::all() {
            await!(self.write_stack_to(other, stack.clone(), typ))?;
        }
        Ok(())
    }
}
