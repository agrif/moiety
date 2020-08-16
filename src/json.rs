use crate::{Format, FormatWrite, Record};

use anyhow::Result;
use smol::io::{AsyncRead, AsyncReadExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JsonFormat(pub bool);

#[async_trait::async_trait(?Send)]
impl<R, I, T> Format<R, I, Record<T>> for JsonFormat
where
    I: AsyncRead + Unpin,
    T: for<'a> serde::Deserialize<'a> + 'static,
{
    fn extension(&self, _res: &R) -> Option<&str> {
        Some(".json")
    }
    async fn parse(&self, _res: &R, input: &mut I) -> Result<Record<T>> {
        let mut contents = Vec::with_capacity(128);
        input.read_to_end(&mut contents).await?;
        Ok(Record(serde_json::from_slice(&contents)?))
    }
}

#[async_trait::async_trait(?Send)]
impl<Fi, R, I, T> FormatWrite<Fi, R, I, Record<T>> for JsonFormat
where
    Fi: Format<R, I, Record<T>>,
    I: AsyncRead + Unpin,
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + 'static,
{
    async fn convert(&self, fmti: &Fi, res: &R, input: &mut I)
                     -> Result<Vec<u8>>
    {
        let data = fmti.parse(res, input).await?;
        if self.0 {
            Ok(serde_json::to_vec_pretty(&*data)?)
        } else {
            Ok(serde_json::to_vec(&*data)?)
        }
    }
}
