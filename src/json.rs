use crate::{
    filesystem::AsyncRead,
    future::*,
};

#[derive(Fail, Debug)]
pub enum JsonError {
    #[fail(display = "{}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "{}", _0)]
    Json(#[cause] serde_json::Error),
}

impl std::convert::From<std::io::Error> for JsonError {
    fn from(err: std::io::Error) -> Self { JsonError::Io(err) }
}

impl std::convert::From<serde_json::Error> for JsonError {
    fn from(err: serde_json::Error) -> Self { JsonError::Json(err) }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct JsonFormat;

impl<F> crate::Format<F> for JsonFormat
where
    F: AsyncRead,
{
    type Error = JsonError;
}

impl<F, R> crate::FormatFor<F, R> for JsonFormat
where
    F: AsyncRead,
    R: crate::ResourceType,
    R::Data: for<'a> serde::Deserialize<'a>,
{
    fn convert<'a>(&'a self, input: F) -> Fut<'a, Result<R::Data, Self::Error>>
    where
        F: 'a,
    {
        fut!({
            let mut contents = Vec::with_capacity(128);
            await!(input.read_until_end_at(0, &mut contents))?;
            Ok(serde_json::from_slice(&contents)?)
        })
    }

    fn extension<'a>(&'a self) -> Option<&'a str> { Some(&".json") }
}

impl<F, R, Fmt> crate::FormatWriteFor<F, R, Fmt> for JsonFormat
where
    F: AsyncRead,
    R: crate::ResourceType,
    R::Data: serde::Serialize,
    Fmt: crate::FormatFor<F, R>,
{
    type WriteError = serde_json::Error;

    fn write<'a>(
        &'a self,
        input: F,
        fmt: &'a Fmt,
    ) -> Fut<
        'a,
        Result<Vec<u8>, crate::ConvertError<Fmt::Error, Self::WriteError>>,
    >
    where
        F: 'a,
        Fmt: 'a,
    {
        fut!({
            let data = await!(fmt.convert(input))
                .map_err(crate::ConvertError::Read)?;
            serde_json::to_vec_pretty(&data).map_err(crate::ConvertError::Write)
        })
    }
}
