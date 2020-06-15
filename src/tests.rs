use crate::jmdict::JMDict;
use crate::kanjidic::Kanjidic;
use std::env;

#[test]
fn jmdict_works() {
    let cwd = env::current_dir().unwrap();
    let jmdict = cwd.join("JMDict.xml");
    let path = jmdict.to_str().unwrap();

    let dict = JMDict::from_file(path).unwrap();
    let _result: Vec<_> = dict
        .filter_gloss(|g| match &g.content {
            Some(t) => t.contains("book"),
            None => false,
        })
        .iter()
        .flat_map(|e| &e.kanji)
        .map(|k| &k.text)
        .collect();
}

#[test]
fn kanjidic_works() {
    let cwd = env::current_dir().unwrap();
    let kanjidic_path = cwd.join("kanjidic2.xml");
    let path = kanjidic_path.to_str().unwrap();

    let dict = Kanjidic::from_file(path).unwrap();
    println!("{:?}", dict);
}
