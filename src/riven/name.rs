use super::Resource;
use crate::{
    filesystem::AsyncRead,
    future::*,
    mhk::{
        deserialize_from,
        deserialize_vec_from,
    },
    FormatFor,
    MhkError,
    MhkFormat,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Name {
    pub unknown: u16,
    pub name: String,
}

impl<R> FormatFor<R, Resource<Vec<Name>>> for MhkFormat
where
    R: AsyncRead,
{
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Vec<Name>, MhkError>>
    where
        R: 'a,
    {
        fut!({
            let mut pos = 0;
            let field_count: u16 = await!(deserialize_from(&input, &mut pos))?;
            let offsets: Vec<u16> = await!(deserialize_vec_from(
                &input,
                &mut pos,
                field_count as usize
            ))?;
            let values: Vec<u16> = await!(deserialize_vec_from(
                &input,
                &mut pos,
                field_count as usize
            ))?;
            let mut ret = Vec::with_capacity(offsets.len());
            for (offs, val) in offsets.iter().zip(values) {
                let mut name = Vec::new();
                await!(input.read_until_at(pos + *offs as u64, 0, &mut name))?;
                ret.push(Name {
                    unknown: val,
                    name: String::from_utf8_lossy(&name[..name.len() - 1])
                        .into_owned(),
                });
            }
            Ok(ret)
        })
    }
}
