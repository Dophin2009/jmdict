use roxmltree;
use std::error;
use std::fmt;
use std::io;
use std::num;

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    Xml(XmlError),
    ParseString(String),
    ParseInt(num::ParseIntError),
    ParseEnum(ParseEnumError),
    ParseLanguage6391(String),
    ParseLanguage6393(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::IO(ref err) => write!(f, "IO error: {}", err),
            ParseError::Xml(ref err) => write!(f, "XML error: {}", err),
            ParseError::ParseString(ref err_str) => write!(f, "Parse error: {}", err_str),
            ParseError::ParseInt(ref err) => write!(f, "Parse error: {}", err),
            ParseError::ParseEnum(ref err) => write!(f, "Parse error: {}", err),
            ParseError::ParseLanguage6391(ref lang) => {
                write!(f, "invalid ISO 639-1 language code: {}", lang)
            }
            ParseError::ParseLanguage6393(ref lang) => {
                write!(f, "invalid ISO 639-3 language code: {}", lang)
            }
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseError::IO(ref err) => Some(err),
            ParseError::Xml(ref err) => Some(err),
            ParseError::ParseInt(ref err) => Some(err),
            ParseError::ParseEnum(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IO(err)
    }
}

impl From<XmlError> for ParseError {
    fn from(err: XmlError) -> Self {
        ParseError::Xml(err)
    }
}

impl From<num::ParseIntError> for ParseError {
    fn from(err: num::ParseIntError) -> Self {
        ParseError::ParseInt(err)
    }
}

impl From<ParseEnumError> for ParseError {
    fn from(err: ParseEnumError) -> Self {
        ParseError::ParseEnum(err)
    }
}

#[derive(Debug)]
pub enum XmlError {
    Roxml(roxmltree::Error),
    MissingTag(String),
    MissingAttr(String),
    MissingText,
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            XmlError::Roxml(ref err) => write!(f, "roxmltree error: {}", err),
            XmlError::MissingTag(ref name) => write!(f, "XML tag missing: {}", name),
            XmlError::MissingAttr(ref name) => write!(f, "XML element attribute missing: {}", name),
            XmlError::MissingText => write!(f, "XML element text missing"),
        }
    }
}

impl error::Error for XmlError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            XmlError::Roxml(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<roxmltree::Error> for XmlError {
    fn from(error: roxmltree::Error) -> Self {
        XmlError::Roxml(error)
    }
}

#[derive(Debug)]
pub struct ParseEnumError {
    value: String,
    valids: Vec<String>,
}

impl ParseEnumError {
    pub fn new(value: &str, valids: Vec<&str>) -> Self {
        ParseEnumError {
            value: value.to_owned(),
            valids: valids.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let valids_join = self.valids.join(", ");
        write!(
            f,
            "{} is not a valid enum value for [{}]",
            self.value, valids_join
        )
    }
}

impl error::Error for ParseEnumError {}
