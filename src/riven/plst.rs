use super::Resource;
use crate::{
    filesystem::AsyncRead,
    future::*,
    mhk::deserialize_u16_table_from,
    FormatFor,
    MhkError,
    MhkFormat,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PictureMeta {
    pub index: u16,
    pub bitmap_id: u16,
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

impl<R> FormatFor<R, Resource<Vec<PictureMeta>>> for MhkFormat
where
    R: AsyncRead,
{
    fn convert<'a>(
        &'a self,
        input: R,
    ) -> Fut<'a, Result<Vec<PictureMeta>, MhkError>>
    where
        R: 'a,
    {
        fut!({
            let mut pos = 0;
            await!(deserialize_u16_table_from(&input, &mut pos))
        })
    }
}
