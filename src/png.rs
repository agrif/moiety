#[derive(Fail, Debug)]
pub enum PngError {
    #[fail(display = "{}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "{}", _0)]
    Encode(#[cause] png::EncodingError),
    #[fail(display = "{}", _0)]
    Decode(#[cause] png::DecodingError),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PngFormat;

impl<F> crate::Format<F> for PngFormat {
    type Error = PngError;
}
