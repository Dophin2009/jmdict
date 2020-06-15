use roxmltree;
use std::error;
use std::fmt;
use std::io;
use std::num;

#[derive(Debug)]
pub enum ParserError {
    File(io::Error),
    XML(roxmltree::Error),
    MissingTag(String),
    MissingAttr(String),
    MissingText,
    InvalidVal(Box<dyn error::Error>),
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
        ParserError::InvalidVal(Box::new(error))
    }
}

impl From<InvalidEnumError> for ParserError {
    fn from(error: InvalidEnumError) -> Self {
        ParserError::InvalidVal(Box::new(error))
    }
}

#[derive(Debug)]
pub struct InvalidEnumError {
    pub value: String,
    pub valids: Vec<String>,
}

impl InvalidEnumError {
    pub fn new(value: &str, valids: &[&str]) -> Self {
        InvalidEnumError {
            value: value.to_owned(),
            valids: valids.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl error::Error for InvalidEnumError {}

impl fmt::Display for InvalidEnumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let valids_join = self.valids.join(", ");
        write!(
            f,
            "{} is not a valid enum value for [{}]",
            self.value, valids_join
        )
    }
}
