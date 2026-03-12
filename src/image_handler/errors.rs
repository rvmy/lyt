use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum DecodedImageError {
    #[error("Decoding failed: {0}")]
    DecodeError(String),
}

#[derive(Debug, Clone, Error)]
pub enum ImageHandlerError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Unsupported image format")]
    UnSupportedFormat,

    #[error("Decoding error:")]
    Decode(#[from] DecodedImageError),
}
impl From<std::io::Error> for ImageHandlerError {
    fn from(e: std::io::Error) -> Self {
        ImageHandlerError::Io(e.to_string())
    }
}
