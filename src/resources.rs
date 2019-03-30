use crate::{ResourceMap, Format, FormatFor, ResourceType};

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

    pub async fn open_raw<'a, R: ResourceType + 'a>(&'a self, stack: M::Stack, typ: R, id: u16) -> Result<M::Handle, E> {
        let handle = await!(self.map.open_raw(stack, typ, id))?;
        Ok(handle)
    }

    pub async fn open<'a, R: ResourceType + 'a>(&'a self, stack: M::Stack, typ: R, id: u16) -> Result<R::Data, E> where F: FormatFor<M::Handle, R> {
        let handle = await!(self.map.open_raw(stack, typ, id))?;
        let res = await!(self.format.convert(handle))?;
        Ok(res)
    }
}
