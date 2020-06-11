use crate::parser;
use std::env;

#[test]
fn test_works() {
    let cwd = env::current_dir().unwrap();
    let jmdict = cwd.join("JMDict.xml");
    let path = jmdict.to_str().unwrap();

    parser::full_parse(path).unwrap();
}
