#[derive(thiserror::Error, Debug)]
pub enum MhkError {
    #[error("Archive has invalid format: {0}")]
    InvalidFormat(&'static str),
    #[error("Resource does not exist: {0:?} {1} {2}")]
    ResourceNotFound(Option<String>, String, u16),
}
