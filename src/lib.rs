pub mod parser;

use std::str::FromStr;

#[derive(Debug)]
pub struct JMDict {
    entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    seq: i32,
    kana: Vec<Kana>,
    kanji: Vec<Kanji>,
}

#[derive(Debug)]
pub struct Kanji {
    text: String,
    pri_ref: Option<PriRef>,
}

#[derive(Debug)]
pub struct Kana {
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
