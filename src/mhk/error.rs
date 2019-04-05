#[derive(Fail, Debug)]
pub enum MhkError {
    #[fail(display = "Archive has invalid format: {}", _0)]
    InvalidFormat(&'static str),
    #[fail(display = "Resource does not exist: {:?} {} {}", _0, _1, _2)]
    ResourceNotFound(Option<&'static str>, &'static str, u16),
    #[fail(display = "{}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "{}", _0)]
    Decode(#[cause] bincode::Error),
    #[fail(display = "bad string: {}", _0)]
    Utf8(#[cause] std::str::Utf8Error),
}

impl std::convert::From<std::io::Error> for MhkError {
    fn from(err: std::io::Error) -> Self { MhkError::Io(err) }
}

impl std::convert::From<bincode::Error> for MhkError {
    fn from(err: bincode::Error) -> Self { MhkError::Decode(err) }
}

impl std::convert::From<std::str::Utf8Error> for MhkError {
    fn from(err: std::str::Utf8Error) -> Self { MhkError::Utf8(err) }
}
