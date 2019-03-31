use crate::future::*;
use super::MhkError;
use super::utility::*;

#[derive(Debug)]
pub struct MhkFormat;

impl<R> crate::Format<R> for MhkFormat where R: crate::AsyncRead {
    type Error = MhkError;
}

// temporary
impl<R> crate::FormatFor<R, crate::Riven<Vec<crate::Name>>> for MhkFormat where R: crate::AsyncRead {
    fn convert<'a>(&'a self, input: R) -> FutureObjResult<'a, Vec<crate::Name>, MhkError> where R: 'a {
        Box::pin((async move || {
            let mut pos = 0;
            let field_count: u16 = await!(deserialize_from(&input, &mut pos))?;
            let offsets: Vec<u16> = await!(deserialize_vec_from(&input, &mut pos, field_count as usize))?;
            let values: Vec<u16> = await!(deserialize_vec_from(&input, &mut pos, field_count as usize))?;
            let mut ret = Vec::with_capacity(offsets.len());
            for (offs, val) in offsets.iter().zip(values) {
                let mut name = Vec::new();
                await!(input.read_until_at(pos + *offs as u64, 0, &mut name))?;
                ret.push(crate::Name {
                    unknown: val,
                    name: String::from_utf8_lossy(&name[..name.len()-1]).into_owned(),
                });
            }
            Ok(ret)
        })())
    }
}
