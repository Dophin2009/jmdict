pub mod parser;

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

#[cfg(test)]
mod test {
    use crate::parser;
    use std::env;

    #[test]
    fn test_works() {
        let cwd = env::current_dir().unwrap();
        let jmdict = cwd.join("JMDict.xml");
        let path = jmdict.to_str().unwrap();

        parser::full_parse(path).unwrap();
    }
}
