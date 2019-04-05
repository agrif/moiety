use crate::{
    future::*,
    FormatFor,
    ResourceType,
};

pub trait ResourceMap {
    type Handle;
    type Error: failure::Fail;
    type Stack;
    fn open_raw<'a, T: ResourceType + 'a, F: FormatFor<Self::Handle, T>>(
        &'a self,
        fmt: &'a F,
        stack: Self::Stack,
        typ: T,
        id: u16,
    ) -> Fut<'a, Result<Self::Handle, Self::Error>>;
}

pub trait ResourceMapList: ResourceMap {
    fn list<'a, T: ResourceType + 'a>(
        &'a self,
        stack: Self::Stack,
        typ: T,
    ) -> Fut<'a, Result<Vec<u16>, Self::Error>>;
}

pub trait ResourceMapWrite: ResourceMap {
    fn write_raw<'a, T: ResourceType + 'a, F: FormatFor<Self::Handle, T>>(
        &'a mut self,
        fmt: &'a F,
        stack: Self::Stack,
        typ: T,
        id: u16,
        data: &'a [u8],
    ) -> Fut<'a, Result<(), Self::Error>>;
}
