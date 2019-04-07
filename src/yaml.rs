use crate::{
    filesystem::AsyncRead,
    future::*,
};

#[derive(Fail, Debug)]
pub enum YamlError {
    #[fail(display = "{}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "{}", _0)]
    Yaml(#[cause] serde_yaml::Error),
}

impl std::convert::From<std::io::Error> for YamlError {
    fn from(err: std::io::Error) -> Self { YamlError::Io(err) }
}

impl std::convert::From<serde_yaml::Error> for YamlError {
    fn from(err: serde_yaml::Error) -> Self { YamlError::Yaml(err) }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct YamlFormat;

impl<F> crate::Format<F> for YamlFormat
where
    F: AsyncRead,
{
    type Error = YamlError;
}

impl<F, R> crate::FormatFor<F, R> for YamlFormat
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
            await!(input.read_until_end(&mut contents))?;
            Ok(serde_yaml::from_slice(&contents)?)
        })
    }

    fn extension<'a>(&'a self) -> Option<&'a str> { Some(&".yaml") }
}

impl<F, R, Fmt> crate::FormatWriteFor<F, R, Fmt> for YamlFormat
where
    F: AsyncRead,
    R: crate::ResourceType,
    R::Data: serde::Serialize,
    Fmt: crate::FormatFor<F, R>,
{
    type WriteError = serde_yaml::Error;

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
            serde_yaml::to_vec(&data).map_err(crate::ConvertError::Write)
        })
    }
}
