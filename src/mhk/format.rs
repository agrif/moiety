use crate::Format;
use crate::filesystem::AsyncRead;
use super::MhkError;

#[derive(Debug)]
pub struct MhkFormat;

impl<R> Format<R> for MhkFormat where R: AsyncRead {
    type Error = MhkError;
}
