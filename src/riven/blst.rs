use crate::future::*;
use crate::filesystem::AsyncRead;
use crate::mhk::deserialize_u16_table_from;
use crate::{MhkError, MhkFormat, FormatFor};
use super::Resource;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ButtonMeta {
    pub index: u16,
    pub enabled: u16,
    pub hotspot_id: u16,
}

impl<R> FormatFor<R, Resource<Vec<ButtonMeta>>> for MhkFormat where R: AsyncRead {
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Vec<ButtonMeta>, MhkError>> where R: 'a {
        fut!({
            let mut pos = 0;
            await!(deserialize_u16_table_from(&input, &mut pos))
        })
    }
}
