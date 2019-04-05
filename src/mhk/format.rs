use super::MhkError;
use crate::{
    filesystem::AsyncRead,
    future::*,
    Format,
    FormatFor,
    FormatWriteFor,
    ResourceType,
};

#[derive(Debug)]
pub struct MhkFormat;

impl<R> Format<R> for MhkFormat
where
    R: AsyncRead,
{
    type Error = MhkError;
}

impl<F, R> FormatWriteFor<F, R, MhkFormat> for MhkFormat
where
    F: AsyncRead,
    R: ResourceType,
    MhkFormat: FormatFor<F, R>,
    <MhkFormat as Format<F>>::Error: From<std::io::Error>,
{
    type WriteError = MhkError;

    fn write<'a>(
        &'a self,
        input: F,
        _fmt: &'a MhkFormat,
    ) -> Fut<
        'a,
        Result<
            Vec<u8>,
            crate::ConvertError<
                <MhkFormat as Format<F>>::Error,
                Self::WriteError,
            >,
        >,
    >
    where
        F: 'a,
    {
        fut!({
            let mut contents = Vec::with_capacity(128);
            await!(input.read_until_end(&mut contents))
                .map_err(|e| crate::ConvertError::Read(e.into()))?;
            Ok(contents)
        })
    }
}
