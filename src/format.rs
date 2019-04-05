use crate::{
    future::*,
    ResourceType,
};

pub trait Format<I> {
    type Error: failure::Fail;
}

pub trait FormatFor<I, R: ResourceType>: Format<I> {
    fn convert<'a>(&'a self, input: I) -> Fut<'a, Result<R::Data, Self::Error>>
    where
        I: 'a;

    fn extension<'a>(&'a self) -> Option<&'a str> { None }
}

#[derive(Fail, Debug)]
pub enum ConvertError<R: failure::Fail, W: failure::Fail> {
    #[fail(display = "Error reading: {}", _0)]
    Read(#[cause] R),
    #[fail(display = "Error writing: {}", _0)]
    Write(#[cause] W),
}

pub trait FormatWriteFor<I, R: ResourceType, F: FormatFor<I, R>> {
    type WriteError: failure::Fail;
    fn write<'a>(
        &'a self,
        input: I,
        fmt: &'a F,
    ) -> Fut<'a, Result<Vec<u8>, ConvertError<F::Error, Self::WriteError>>>
    where
        I: 'a,
        F: 'a;
}
