use crate::errors::ParseError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Tatoeba {
    pub entries: Vec<Sentence>,
}

pub struct Sentence {
    pub content: String,
    pub language: String,
}

impl Tatoeba {
    pub fn from_file_jp<P: AsRef<Path>>(filepath: P) -> Result<Self, ParseError> {
        Tatoeba::from_file(filepath, Some(|s: &str| s == "jpn"))
    }

    pub fn from_file<F, P: AsRef<Path>>(
        filepath: P,
        language_filter: Option<F>,
    ) -> Result<Self, ParseError>
    where
        F: Fn(&str) -> bool,
    {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        let language_filter: Box<dyn Fn(&str) -> bool> = match language_filter {
            Some(f) => Box::new(f),
            None => Box::new(|_| true),
        };

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<_> = line.splitn(3, "\t").collect();

            let language: &str = parts.get(1).ok_or(ParseError::ParseString(
                "sentence language value not found".to_owned(),
            ))?;
            if !language_filter(language) {
                continue;
            }

            let content: &str = parts.get(2).ok_or(ParseError::ParseString(
                "sentence value not found".to_owned(),
            ))?;

            entries.push(Sentence {
                language: language.to_owned(),
                content: content.to_owned(),
            });
        }

        Ok(Tatoeba { entries })
    }
}
