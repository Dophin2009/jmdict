use roxmltree;
use std::io;
use std::num;

#[derive(Debug)]
pub enum ParserError {
    File(io::Error),
    XML(roxmltree::Error),
    MissingTag(String),
    MissingText,
    ParseInt(num::ParseIntError),
}

impl From<io::Error> for ParserError {
    fn from(error: io::Error) -> Self {
        ParserError::File(error)
    }
}

impl From<roxmltree::Error> for ParserError {
    fn from(error: roxmltree::Error) -> Self {
        ParserError::XML(error)
    }
}

impl From<num::ParseIntError> for ParserError {
    fn from(error: num::ParseIntError) -> Self {
        ParserError::ParseInt(error)
    }
}
