use crate::{Entry, JMDict, Kana, Kanji, PriRef};
use roxmltree::{Document, Node};
use std::fs;
use std::io;
use std::num;
use std::str::FromStr;

const SEQ: &str = "ent_seq";
const K_ELE: &str = "k_ele";
const KEB: &str = "keb";
const KE_PRI: &str = "ke_pri";
const R_ELE: &str = "r_ele";
const REB: &str = "reb";
const RE_PRI: &str = "re_pri";

#[inline]
pub fn full_parse(filepath: &str) -> Result<JMDict, ParserError> {
    let contents = read_file(filepath)?;
    let doc = Document::parse(&contents)?;

    let entries: Vec<_> = doc
        .root_element()
        .children()
        .filter(|n| n.is_element())
        .map(|n| parse_entry(n))
        .collect::<Result<Vec<_>, _>>()?;

    return Ok(JMDict { entries });
}

#[inline]
fn parse_entry(n: Node) -> Result<Entry, ParserError> {
    let mut kana = Vec::new();
    let mut kanji = Vec::new();

    let seq: i32 = {
        let seq_text = find_child_tag(n, SEQ).and_then(|t| t.text());

        match seq_text {
            Some(t) => t.parse()?,
            // None => return Err(ParserError::MissingTag(SEQ.to_owned())),
            None => 0,
        }
    };

    for c in n.children() {
        let tag = c.tag_name().name();
        let text = c.text();

        if text == None {
            continue;
        }

        if tag == K_ELE {
            kanji.push(parse_kanji(c)?);
        }

        if tag == R_ELE {
            kana.push(parse_kana(c)?)
        }
    }

    Ok(Entry { seq, kana, kanji })
}

#[inline]
fn parse_kanji(n: Node) -> Result<Kanji, ParserError> {
    let keb_node = find_child_tag(n, KEB).ok_or(ParserError::MissingTag(KEB.to_owned()))?;
    let keb = keb_node.text().ok_or(ParserError::MissingText)?;

    let ke_pri = find_child_tag(n, KE_PRI)
        .and_then(|k| k.text())
        .and_then(|t| PriRef::from_str(t).ok());

    Ok(Kanji {
        text: keb.to_owned(),
        pri_ref: ke_pri,
    })
}

#[inline]
fn parse_kana(n: Node) -> Result<Kana, ParserError> {
    let reb_node = find_child_tag(n, REB).ok_or(ParserError::MissingTag(REB.to_owned()))?;
    let reb = reb_node.text().ok_or(ParserError::MissingText)?;

    let re_pri = find_child_tag(n, RE_PRI)
        .and_then(|r| r.text())
        .and_then(|t| PriRef::from_str(t).ok());

    Ok(Kana {
        text: reb.to_owned(),
        pri_ref: re_pri,
    })
}

#[inline]
fn find_child_tag<'a>(n: Node<'a, 'a>, tag_name: &str) -> Option<Node<'a, 'a>> {
    n.children().find(|c| c.tag_name().name() == tag_name)
}

fn read_file(filepath: &str) -> Result<String, io::Error> {
    fs::read_to_string(filepath)
}

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
