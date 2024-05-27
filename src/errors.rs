use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("Expected file to start with 'IDAT' but it started with {actual}")]
    InvalidHeader { actual: String },
    #[error("Expected valid field code but it was {actual}")]
    InvalidFieldType { actual: u16 },
    #[error(transparent)]
    Io(#[from] io::Error),
}
