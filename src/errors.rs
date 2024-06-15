use std::io;
use thiserror::Error;

use crate::fields::FieldType;

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("Expected file to start with 'IDAT' but it started with {actual}")]
    InvalidHeader { actual: String },
    #[error("Expected valid field code but it was {actual}")]
    InvalidFieldType { actual: u16 },
    #[error("Record does not contain field")]
    MissingField { field: FieldType },
    #[error("Field not iterable")]
    FieldNotIterable,
    #[error(transparent)]
    Io(#[from] io::Error),
}
