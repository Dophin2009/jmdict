use roxmltree::{Document, Node, NS_XML_URI};
use std::borrow::Cow;
use std::fs;
use std::io;
use std::num;
use std::str::FromStr;

#[derive(Debug)]
pub struct JMDict {
    entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    seq: i32,
    reading: Vec<Reading>,
    kanji: Vec<Kanji>,
    sense: Vec<Sense>,
}

#[derive(Debug)]
pub struct Kanji {
    text: String,
    pri_ref: Option<PriRef>,
}

#[derive(Debug)]
pub struct Reading {
    text: String,
    pri_ref: Option<PriRef>,
}

#[derive(Debug)]
pub enum PriRef {
    News1,
    News2,
    Ichi1,
    Ichi2,
    Spec1,
    Spec2,
    Gai1,
    Gai2,
    NF(i32),
}

impl FromStr for PriRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match &*s.to_lowercase() {
            "news1" => Ok(PriRef::News1),
            "news2" => Ok(PriRef::News2),
            "ichi1" => Ok(PriRef::Ichi1),
            "ichi2" => Ok(PriRef::Ichi2),
            "spec1" => Ok(PriRef::Spec1),
            "spec2" => Ok(PriRef::Spec2),
            "gai1" => Ok(PriRef::Gai1),
            "gai2" => Ok(PriRef::Gai2),
            x => {
                if x.starts_with("nf") {
                    let xn: String = x.chars().skip(2).collect();
                    match xn.parse() {
                        Ok(n) => Ok(PriRef::NF(n)),
                        Err(_) => Err(()),
                    }
                } else {
                    Err(())
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Sense {
    restrict_reading: Vec<String>,
    restrict_kanji: Vec<String>,
    cross_refs: Vec<String>,
    gloss: Vec<Gloss>,
    antonyms: Vec<String>,
    pos: Vec<String>,
    fields: Vec<String>,
    misc: Vec<String>,
    source_lang: Vec<LSource>,
    dialects: Vec<String>,
    info: Vec<String>,
}

#[derive(Debug)]
pub struct Gloss {
    content: Option<String>,
    lang: String,
    gender: Option<String>,
    typ: Option<String>,
}

#[derive(Debug)]
pub struct LSource {
    content: Option<String>,
    lang: String,
    // The full attribute indicates whether the source language
    // fully or partially describes the source word or phrase of the
    // loanword. If absent, it will have the implied value of "full".
    full: bool,
    // The wasei attribute indicates that the Japanese word
    // has been constructed from words in the source language, and
    // not from an actual phrase in that language. Most commonly used to
    // indicate "waseieigo".
    wasei: bool,
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

macro_rules! const_strs {
    ( $( $id:ident : $val:expr ),* $(,)? ) => {
        $(const $id: &str = $val;)*
    };
}

const_strs!(
    SEQ: "ent_seq",

    KANJI_ELE: "k_ele",
    KANJI_TEXT: "keb",
    KANJI_PRI: "ke_pri",

    READING_ELE: "r_ele",
    READING_TEXT: "reb",
    READING_PRI: "re_pri",

    SENSE: "sense",
    RESTRICT_KANJI: "stagk",
    RESTRICT_READING: "stagr",
    CROSS_REF: "xref",
    ANTONYM: "ant",
    POS: "pos",
    FIELD: "field",
    MISC: "misc",
    DIALECT: "dial",
    INFO: "s_inf",

    LSOURCE: "lsource",
    LSOURCE_LANG_SUFFIX: "lang",
    LSOURCE_LANG_DEF: "eng",
    LSOURCE_TYPE: "ls_type",
    LSOURCE_WASEI: "ls_wasei",

    GLOSS: "gloss",
    GLOSS_LANG_SUFFIX: "lang",
    GLOSS_LANG_DEFAULT: "eng",
    GLOSS_GENDER: "g_gend",
    GLOSS_TYPE: "g_type"
);

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
    let mut reading = Vec::new();
    let mut kanji = Vec::new();
    let mut sense = Vec::new();

    let seq: i32 = {
        let seq_text = find_child_tag(n, SEQ).and_then(|t| t.text());

        match seq_text {
            Some(t) => t.parse()?,
            None => return Err(ParserError::MissingTag(SEQ.to_owned())),
        }
    };

    for c in n.children() {
        let tag = c.tag_name().name();
        let text = c.text();

        if text == None {
            continue;
        }

        if tag == KANJI_ELE {
            kanji.push(parse_kanji(c)?);
        }

        if tag == READING_ELE {
            reading.push(parse_reading(c)?)
        }

        if tag == SENSE {
            sense.push(parse_sense(c)?)
        }
    }

    Ok(Entry {
        seq,
        reading,
        kanji,
        sense,
    })
}

#[inline]
fn parse_reading(n: Node) -> Result<Reading, ParserError> {
    let reb_node =
        find_child_tag(n, READING_TEXT).ok_or(ParserError::MissingTag(READING_TEXT.to_owned()))?;
    let reb = reb_node.text().ok_or(ParserError::MissingText)?;

    let re_pri = find_child_tag(n, READING_PRI)
        .and_then(|r| r.text())
        .and_then(|t| t.parse().ok());

    Ok(Reading {
        text: reb.to_owned(),
        pri_ref: re_pri,
    })
}

#[inline]
fn parse_kanji(n: Node) -> Result<Kanji, ParserError> {
    let keb_node =
        find_child_tag(n, KANJI_TEXT).ok_or(ParserError::MissingTag(KANJI_TEXT.to_owned()))?;
    let keb = keb_node.text().ok_or(ParserError::MissingText)?;

    let ke_pri = find_child_tag(n, KANJI_PRI)
        .and_then(|k| k.text())
        .and_then(|t| t.parse().ok());

    Ok(Kanji {
        text: keb.to_owned(),
        pri_ref: ke_pri,
    })
}

#[inline]
fn parse_sense(n: Node) -> Result<Sense, ParserError> {
    let mut sense = Sense {
        restrict_reading: Vec::new(),
        restrict_kanji: Vec::new(),
        cross_refs: Vec::new(),
        antonyms: Vec::new(),
        pos: Vec::new(),
        fields: Vec::new(),
        misc: Vec::new(),
        gloss: Vec::new(),
        source_lang: Vec::new(),
        dialects: Vec::new(),
        info: Vec::new(),
    };

    for c in n.children() {
        let tag = c.tag_name().name();
        let text = get_node_text(c);
        match tag {
            RESTRICT_READING => sense.restrict_reading.push(text?.into_owned()),
            RESTRICT_KANJI => sense.restrict_kanji.push(text?.into_owned()),
            CROSS_REF => sense.cross_refs.push(text?.into_owned()),
            ANTONYM => sense.antonyms.push(text?.into_owned()),
            POS => sense.pos.push(text?.into_owned()),
            FIELD => sense.fields.push(text?.into_owned()),
            MISC => sense.misc.push(text?.into_owned()),
            DIALECT => sense.dialects.push(text?.into_owned()),
            INFO => sense.info.push(text?.into_owned()),
            LSOURCE => {
                let content = text.ok().and_then(|t| Some(t.into_owned()));
                let lang = c
                    .attribute(ns_xml_attr(LSOURCE_LANG_SUFFIX))
                    .unwrap_or_else(|| LSOURCE_LANG_DEF)
                    .to_owned();
                let full = c.attribute(LSOURCE_TYPE).map_or(true, |_| false);
                let wasei = c.attribute(LSOURCE_WASEI).map_or(false, |_| true);

                sense.source_lang.push(LSource {
                    content,
                    lang,
                    full,
                    wasei,
                });
            }
            GLOSS => {
                let content = text.ok().and_then(|t| Some(t.into_owned()));
                let lang = c
                    .attribute(ns_xml_attr(GLOSS_LANG_SUFFIX))
                    .unwrap_or_else(|| GLOSS_LANG_DEFAULT)
                    .to_owned();
                let gender = c.attribute(GLOSS_GENDER).and_then(|g| Some(g.to_owned()));
                let typ = c.attribute(GLOSS_TYPE).and_then(|t| Some(t.to_owned()));
                sense.gloss.push(Gloss {
                    content,
                    lang,
                    gender,
                    typ,
                });
            }
            _ => {}
        }
    }

    Ok(sense)
}

#[inline]
fn find_child_tag<'a>(n: Node<'a, 'a>, tag_name: &str) -> Option<Node<'a, 'a>> {
    n.children().find(|c| c.tag_name().name() == tag_name)
}

#[inline]
fn get_node_text<'a>(n: Node<'a, 'a>) -> Result<Cow<'a, str>, ParserError> {
    n.text().ok_or(ParserError::MissingText).map(|t| t.into())
}

#[inline]
fn ns_xml_attr(attr: &str) -> (&str, &str) {
    (NS_XML_URI, attr)
}

fn read_file(filepath: &str) -> Result<String, io::Error> {
    fs::read_to_string(filepath)
}
