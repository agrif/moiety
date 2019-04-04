use crate::future::*;
use crate::filesystem::AsyncRead;
use crate::mhk::deserialize_from;
use crate::{MhkError, MhkFormat, FormatFor};
use super::{Resource, Event, Command, deserialize_handlers};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Card {
    pub name_rec: i16,
    pub zip_mode_place: u16,
    pub script: std::collections::HashMap<Event, Vec<Command>>,
}

impl<R> FormatFor<R, Resource<Card>> for MhkFormat where R: AsyncRead {
    fn convert<'a>(&'a self, input: R) -> Fut<'a, Result<Card, MhkError>> where R: 'a {
        fut!({
            let mut pos = 0;
            let name_rec = await!(deserialize_from(&input, &mut pos))?;
            let zip_mode_place = await!(deserialize_from(&input, &mut pos))?;
            let script = await!(deserialize_handlers(&input, &mut pos))?;
            Ok(Card {
                name_rec,
                zip_mode_place,
                script,
            })
        })
    }
}
