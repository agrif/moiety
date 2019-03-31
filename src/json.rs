use crate::future::*;

#[derive(Fail, Debug)]
pub enum JsonError {
    #[fail(display = "{}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "{}", _0)]
    Json(#[cause] serde_json::Error),
}

impl std::convert::From<std::io::Error> for JsonError {
    fn from(err: std::io::Error) -> Self {
        JsonError::Io(err)
    }
}

impl std::convert::From<serde_json::Error> for JsonError {
    fn from(err: serde_json::Error) -> Self {
        JsonError::Json(err)
    }
}

pub struct JsonFormat;

impl<F> crate::Format<F> for JsonFormat where F: crate::AsyncRead {
    type Error = std::io::Error;
}

impl<F, R> crate::FormatFor<F, R> for JsonFormat where F: crate::AsyncRead, R: crate::ResourceType, R::Data: for<'a> serde::Deserialize<'a> {
    fn convert<'a>(&'a self, input: F) -> FutureObjResult<'a, R::Data, Self::Error> where F: 'a {
        Box::pin((async move || {
            let mut contents = Vec::with_capacity(128);
            await!(input.read_until_end(&mut contents))?;
            Ok(serde_json::from_slice(&contents)?)
        })())
    }
    
}

impl<F, R, Fmt> crate::FormatWriteFor<F, R, Fmt> for JsonFormat where F: crate::AsyncRead, R: crate::ResourceType, R::Data: serde::Serialize, Fmt: crate::FormatFor<F, R> {
    type WriteError = serde_json::Error;
    fn write<'a>(&'a self, input: F, fmt: &'a Fmt) -> FutureObjResult<'a, Vec<u8>, crate::ConvertError<Fmt::Error, Self::WriteError>> where F: 'a, Fmt: 'a {
        Box::pin((async move || {
            let data = await!(fmt.convert(input)).map_err(crate::ConvertError::Read)?;
            serde_json::to_vec_pretty(&data).map_err(crate::ConvertError::Write)
        })())
    }
}