use crate::{Entry, JMDict, Kana, Kanji, PriRef};
use roxmltree::Document;
use std::fs;
use std::io;
use std::str::FromStr;

const SEQ: &str = "ent_seq";
const K_ELE: &str = "k_ele";
const KEB: &str = "keb";
const KE_PRI: &str = "ke_pri";
const R_ELE: &str = "r_ele";
const REB: &str = "reb";
const RE_PRI: &str = "re_pri";

pub fn full_parse(filepath: &str) -> Result<JMDict, ParserError> {
    let contents = read_file(filepath)?;
    let doc = Document::parse(&contents)?;
    let entries: Vec<_> = doc
        .descendants()
        .map(|n| parse_entry(n))
        .filter_map(|x| x.ok())
        .collect();

    return Ok(JMDict { entries });
}

fn parse_entry(n: roxmltree::Node) -> Result<Entry, ParserError> {
    let mut kana = Vec::new();
    let mut kanji = Vec::new();

    macro_rules! unwrap_text {
        ($text:expr) => {
            match $text {
                Some(s) => s,
                None => return Err(ParserError::MissingText),
            };
        };
    }

    let seq: i32 = {
        let seq_tag = n.children().find(|p| p.tag_name().name() == SEQ);
        let seq_text = match seq_tag {
            Some(t) => t.text(),
            None => return Err(ParserError::MissingTag(SEQ.to_owned())),
        };

        match unwrap_text!(seq_text).parse() {
            Ok(i) => i,
            Err(_) => return Err(ParserError::MissingText),
        }
    };

    for elem in n.children() {
        let tag = elem.tag_name().name();

        let text = elem.text();

        if text == None {
            continue;
        }

        if tag == K_ELE {
            let keb = match elem.children().find(|c| c.tag_name().name() == KEB) {
                Some(k) => unwrap_text!(k.text()),
                None => return Err(ParserError::MissingTag(KEB.to_owned())),
            };

            let ke_pri = elem
                .children()
                .find(|c| c.tag_name().name() == KE_PRI)
                .and_then(|k| k.text())
                .and_then(|t| PriRef::from_str(t).ok());

            kanji.push(Kanji {
                text: keb.to_owned(),
                pri_ref: ke_pri,
            });
        }

        if tag == R_ELE {
            let reb = match elem.first_element_child() {
                Some(r) => unwrap_text!(r.text()),
                None => return Err(ParserError::MissingTag(REB.to_owned())),
            };

            let re_pri = elem
                .children()
                .find(|c| c.tag_name().name() == RE_PRI)
                .and_then(|r| r.text())
                .and_then(|t| PriRef::from_str(t).ok());

            kana.push(Kana {
                text: reb.to_owned(),
                pri_ref: re_pri,
            });
        }
    }

    Ok(Entry { seq, kana, kanji })
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
    InvalidValue(String, String),
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
