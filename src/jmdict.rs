use crate::errors::{InvalidEnumError, ParserError};
use crate::util::{self, find_child_tag, find_child_tag_err, get_node_text, ns_xml_attr};
use roxmltree::{Document, Node};

#[derive(Debug)]
pub struct JMDict {
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub seq: i32,
    pub reading: Vec<Reading>,
    pub kanji: Vec<Kanji>,
    pub sense: Vec<Sense>,
}

#[derive(Debug)]
pub struct Kanji {
    pub text: String,
    pub pri_ref: Option<PriRef>,
}

#[derive(Debug)]
pub struct Reading {
    pub text: String,
    pub pri_ref: Option<PriRef>,
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

#[derive(Debug)]
pub struct Sense {
    pub restrict_reading: Vec<String>,
    pub restrict_kanji: Vec<String>,
    pub cross_refs: Vec<String>,
    pub gloss: Vec<Gloss>,
    pub antonyms: Vec<String>,
    pub pos: Vec<String>,
    pub fields: Vec<String>,
    pub misc: Vec<String>,
    pub source_lang: Vec<LSource>,
    pub dialects: Vec<String>,
    pub info: Vec<String>,
}

#[derive(Debug)]
pub struct Gloss {
    pub content: Option<String>,
    pub lang: String,
    pub gender: Option<String>,
    pub typ: Option<String>,
}

#[derive(Debug)]
pub struct LSource {
    pub content: Option<String>,
    pub lang: String,
    // The full attribute indicates whether the source language
    // fully or partially describes the source word or phrase of the
    // loanword. If absent, it will have the implied value of "full".
    pub full: bool,
    // The wasei attribute indicates that the Japanese word
    // has been constructed from words in the source language, and
    // not from an actual phrase in that language. Most commonly used to
    // indicate "waseieigo".
    pub wasei: bool,
}

impl JMDict {
    pub fn find_seq(&self, seq: i32) -> Option<&Entry> {
        self.entries.iter().find(|e| e.seq == seq)
    }

    pub fn filter<F>(&self, predicate: F) -> Vec<&Entry>
    where
        F: Fn(&Entry) -> bool,
    {
        self.entries.iter().filter(|e| predicate(e)).collect()
    }

    pub fn filter_reading<F>(&self, predicate: F) -> Vec<&Entry>
    where
        F: Fn(&Reading) -> bool,
    {
        self.entries
            .iter()
            .filter(|e| e.reading.iter().any(|r| predicate(r)))
            .collect()
    }

    pub fn filter_kanji<F>(&self, predicate: F) -> Vec<&Entry>
    where
        F: Fn(&Kanji) -> bool,
    {
        self.entries
            .iter()
            .filter(|e| e.kanji.iter().any(|k| predicate(k)))
            .collect()
    }

    pub fn filter_gloss<F>(&self, predicate: F) -> Vec<&Entry>
    where
        F: Fn(&Gloss) -> bool,
    {
        self.entries
            .iter()
            .filter(|e| e.sense.iter().flat_map(|s| &s.gloss).any(|g| predicate(g)))
            .collect()
    }

    pub fn search(&self, phrase: &str) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|e| {
                e.reading.iter().any(|r| r.text == phrase)
                    || e.kanji.iter().any(|k| k.text == phrase)
            })
            .collect()
    }

    pub fn antonyms(&self, entry: &Entry) -> Vec<&Entry> {
        let ant = entry.sense.iter().flat_map(|s| &s.antonyms);
        ant.flat_map(|a| self.search(&a)).collect()
    }
}

impl JMDict {
    pub fn from_file(filepath: &str) -> Result<Self, ParserError> {
        let contents = util::read_file(filepath)?;
        let doc = Document::parse(&contents)?;

        let entries: Vec<_> = doc
            .root_element()
            .children()
            .filter(|n| n.is_element())
            .map(|n| parse_entry(n))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(JMDict { entries })
    }
}

const_strs!(
    SEQ: "ent_seq",
    KANJI_ELE: "k_ele",
    READING_ELE: "r_ele",
    SENSE: "sense",
);

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

const_strs!(
    READING_TEXT: "reb",
    READING_PRI: "re_pri",
);

fn parse_reading(n: Node) -> Result<Reading, ParserError> {
    let reb_node = find_child_tag_err(n, READING_TEXT)?;
    let reb = get_node_text(reb_node)?;

    let re_pri = find_child_tag(n, READING_PRI)
        .and_then(|r| r.text())
        .and_then(|t| parse_pri_ref(t).ok());

    Ok(Reading {
        text: reb.into_owned(),
        pri_ref: re_pri,
    })
}

const_strs!(
    KANJI_TEXT: "keb",
    KANJI_PRI: "ke_pri",
);

fn parse_kanji(n: Node) -> Result<Kanji, ParserError> {
    let keb_node = find_child_tag_err(n, KANJI_TEXT)?;
    let keb = get_node_text(keb_node)?;

    let ke_pri = find_child_tag(n, KANJI_PRI)
        .and_then(|k| k.text())
        .and_then(|t| parse_pri_ref(t).ok());

    Ok(Kanji {
        text: keb.into_owned(),
        pri_ref: ke_pri,
    })
}

fn parse_pri_ref(t: &str) -> Result<PriRef, ParserError> {
    match t {
        "news1" => Ok(PriRef::News1),
        "news2" => Ok(PriRef::News2),
        "ichi1" => Ok(PriRef::Ichi1),
        "ichi2" => Ok(PriRef::Ichi2),
        "spec1" => Ok(PriRef::Spec1),
        "spec2" => Ok(PriRef::Spec2),
        "gai1" => Ok(PriRef::Gai1),
        "gai2" => Ok(PriRef::Gai2),
        x => {
            let valids = &[
                "news1", "news2", "ichi1", "ichi2", "spec1", "spec2", "gai1", "gai2", "nfxx",
            ];
            if x.starts_with("nf") {
                let xn: String = x.chars().skip(2).collect();
                match xn.parse() {
                    Ok(n) => Ok(PriRef::NF(n)),
                    Err(err) => Err(err.into()),
                }
            } else {
                Err(InvalidEnumError::new(x, valids).into())
            }
        }
    }
}

const_strs!(
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
