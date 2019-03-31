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
