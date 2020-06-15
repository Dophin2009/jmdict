use crate::errors::{ParseError, XmlError};
use roxmltree::{ExpandedName, Node, NS_XML_URI};
use std::borrow::Cow;
use std::fs;
use std::io;

macro_rules! const_strs {
    ( $( $id:ident : $val:expr ),* $(,)? ) => {
        $(const $id: &str = $val;)*
    };
}

pub fn find_child_tag<'a>(n: Node<'a, 'a>, tag_name: &str) -> Option<Node<'a, 'a>> {
    n.children().find(|c| c.tag_name().name() == tag_name)
}

pub fn find_child_tag_err<'a>(n: Node<'a, 'a>, tag_name: &str) -> Result<Node<'a, 'a>, ParseError> {
    find_child_tag(n, tag_name).ok_or(XmlError::MissingTag(tag_name.to_owned()).into())
}

pub fn get_node_attr<'a>(n: Node<'a, 'a>, attr_name: &str) -> Result<Cow<'a, str>, ParseError> {
    n.attribute(attr_name)
        .ok_or(XmlError::MissingAttr(attr_name.to_owned()).into())
        .map(|t| t.into())
}

pub fn get_node_text<'a>(n: Node<'a, 'a>) -> Result<Cow<'a, str>, ParseError> {
    n.text()
        .ok_or(XmlError::MissingText.into())
        .map(|t| t.into())
}

pub fn ns_xml_attr(attr: &str) -> ExpandedName {
    (NS_XML_URI, attr).into()
}

pub fn read_file(filepath: &str) -> Result<String, io::Error> {
    fs::read_to_string(filepath)
}
