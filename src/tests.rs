use crate::jmdict::JMDict;
use std::env;

#[test]
fn jmdict_works() {
    let cwd = env::current_dir().unwrap();
    let jmdict = cwd.join("JMDict.xml");
    let path = jmdict.to_str().unwrap();

    JMDict::from_file(path).unwrap();
}
