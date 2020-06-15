use crate::errors::{InvalidEnumError, ParserError};
use crate::util;
use roxmltree::{Document, Node};

#[derive(Debug)]
pub struct Kanjidic {
    pub file_version: i32,
    pub database_version: String,
    pub creation_date: String,

    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub literal: String,
    pub codepoints: Vec<Codepoint>,

    pub reading_meanings: Vec<ReadingMeaning>,
    pub nanori_readings: Vec<String>,

    pub radicals: Vec<Radical>,
    pub stroke_count: i32,
    pub stroke_miscounts: Vec<i32>,

    pub grade: Option<Grade>,
    pub freq: Option<i32>,
    pub old_jlpt: Option<i32>,
    pub dic_refs: Vec<DicRef>,
}

#[derive(Debug)]
pub struct Codepoint {
    pub standard: String,
    pub value: String,
}

#[derive(Debug)]
pub struct ReadingMeaning {
    pub readings: Vec<Reading>,
    pub meanings: Vec<Meaning>,
}

#[derive(Debug)]
pub struct Reading {
    pub value: String,
    pub typ: ReadingType,
}

#[derive(Debug)]
pub enum ReadingType {
    Pinyin,
    KoreanR,
    KoreanH,
    Vietnam,
    Onyomi(bool, OnyomiType),
    Kunyomi(bool),
}

#[derive(Debug)]
pub enum OnyomiType {
    Kan,
    Go,
    Tou,
    Kanyou,
    None,
}

#[derive(Debug)]
pub struct Meaning {
    pub content: String,
    pub language: String,
}

#[derive(Debug)]
pub struct Radical {
    pub classification: RadicalType,
    pub value: i32,
}

#[derive(Debug)]
pub enum RadicalType {
    Classical,
    NelsonC,
}

#[derive(Debug)]
pub enum Grade {
    Kyouiku(i32),
    Jouyou,
    Jinmeiyou,
    JouyouVariant,
}

#[derive(Debug)]
pub enum DicRef {
    NelsonC(String),
    NelsonN(String),
    HalpernNJECD(String),
    HalpernKKD(String),
    HalpernKKLD(String),
    HalpernKKLD2(String),
    Heisig(String),
    Heisig6(String),
    Gakken(String),
    OneillNames(String),
    OneillKK(String),
    NeillKK(String),
    Moro(String, Option<i32>, Option<i32>),
    Henshall(String),
    SHKK(String),
    SHKK2(String),
    Sakade(String),
    JFCards(String),
    Henshall3(String),
    TuttCards(String),
    Crowley(String),
    InContext(String),
    BusyPeople(String),
    KodanshaCompact(String),
    Maniette(String),
}

const_strs!(
    ROOT: "kanjidic2",
    HEADER: "header",
    CHARACTER: "character"
);

impl Kanjidic {
    pub fn from_file(filepath: &str) -> Result<Self, ParserError> {
        let contents = util::read_file(filepath)?;
        let doc = Document::parse(&contents)?;
        let root = util::find_child_tag_err(doc.root(), ROOT)?;

        let header = util::find_child_tag_err(root, HEADER)?;
        let (file_version, database_version, creation_date) = parse_header(header)?;

        let entries: Vec<_> = root
            .children()
            .filter(|c| c.is_element() && c.tag_name().name() == CHARACTER)
            .map(|c| parse_entry(c))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Kanjidic {
            file_version,
            database_version,
            creation_date,
            entries,
        })
    }
}

const_strs!(
    FILE_VERSION: "file_version",
    DATABASE_VERSION: "database_version",
    CREATION_DATE: "date_of_creation",
);

fn parse_header(header: Node) -> Result<(i32, String, String), ParserError> {
    let file_version_node = util::find_child_tag_err(header, FILE_VERSION)?;
    let file_version = util::get_node_text(file_version_node)?.parse()?;

    let database_version_node = util::find_child_tag_err(header, DATABASE_VERSION)?;
    let database_version = util::get_node_text(database_version_node)?.into_owned();

    let creation_date_node = util::find_child_tag_err(header, CREATION_DATE)?;
    let creation_date = util::get_node_text(creation_date_node)?.into_owned();

    Ok((file_version, database_version, creation_date))
}

const_strs!(
    LITERAL: "literal",

    CODEPOINT_GROUP: "codepoint",
    CODEPOINT: "cp_value",
    CODEPOINT_TYPE: "cp_type",

    RADICAL_GROUP: "radical",
    RADICAL: "rad_value",
    RADICAL_TYPE: "rad_type",

    MISC: "misc",

    DIC_REF_GROUP: "dic_number",

    READING_GROUP: "reading_meaning",
);

fn parse_entry(n: Node) -> Result<Entry, ParserError> {
    let mut literal_op: Option<String> = None;
    let mut codepoints_op: Option<Vec<Codepoint>> = None;
    let mut radicals_op: Option<Vec<Radical>> = None;
    let mut misc_op: Option<Misc> = None;
    let mut dic_refs_op: Option<Vec<DicRef>> = None;
    let mut readings_meanings_op: Option<Vec<ReadingMeaning>> = None;
    let mut nanori_op: Option<Vec<String>> = None;

    for c in n.children() {
        let tag_name = c.tag_name().name();
        match tag_name {
            LITERAL => literal_op = Some(util::get_node_text(c)?.into()),
            CODEPOINT_GROUP => {
                codepoints_op = Some(
                    c.children()
                        .filter(|cc| cc.tag_name().name() == CODEPOINT)
                        .map(|cc| parse_codepoint(cc))
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            RADICAL_GROUP => {
                radicals_op = Some(
                    c.children()
                        .filter(|cc| cc.tag_name().name() == RADICAL)
                        .map(|cc| parse_radical(cc))
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            MISC => {
                misc_op = Some(parse_misc(c)?);
            }
            DIC_REF_GROUP => dic_refs_op = Some(parse_dic_ref_group(c)?),
            READING_GROUP => {
                let (readings, nanori_readings) = parse_reading_meanings(c)?;
                readings_meanings_op = Some(readings);
                nanori_op = Some(nanori_readings);
            }
            _ => {}
        }
    }

    println!("{}", literal_op.clone().unwrap());

    let misc = misc_op.ok_or(ParserError::MissingTag(MISC.to_owned()))?;

    Ok(Entry {
        literal: literal_op.ok_or(ParserError::MissingTag(LITERAL.to_owned()))?,
        codepoints: codepoints_op.ok_or(ParserError::MissingTag(CODEPOINT_GROUP.to_owned()))?,
        radicals: radicals_op.ok_or(ParserError::MissingTag(RADICAL_GROUP.to_owned()))?,
        grade: misc.grade,
        stroke_count: misc.stroke_count,
        stroke_miscounts: misc.stroke_miscounts,
        freq: misc.freq,
        old_jlpt: misc.old_jlpt,
        dic_refs: dic_refs_op.unwrap_or(Vec::new()),
        reading_meanings: readings_meanings_op.unwrap_or(Vec::new()),
        nanori_readings: nanori_op.unwrap_or(Vec::new()),
    })
}

fn parse_codepoint(n: Node) -> Result<Codepoint, ParserError> {
    let standard = util::get_node_attr(n, CODEPOINT_TYPE)?.into_owned();
    let value = util::get_node_text(n)?.into_owned();

    Ok(Codepoint { standard, value })
}

fn parse_radical(n: Node) -> Result<Radical, ParserError> {
    let classification_attr = util::get_node_attr(n, RADICAL_TYPE)?;
    let classification = match classification_attr.as_ref() {
        "classical" => RadicalType::Classical,
        "nelson_c" => RadicalType::NelsonC,
        _ => {
            let valids = &["classical", "nelson_c"];
            return Err(InvalidEnumError::new(classification_attr.as_ref(), valids).into());
        }
    };
    let value = util::get_node_text(n)?.parse()?;

    Ok(Radical {
        classification,
        value,
    })
}

struct Misc {
    stroke_count: i32,
    stroke_miscounts: Vec<i32>,

    grade: Option<Grade>,
    freq: Option<i32>,
    old_jlpt: Option<i32>,
}

const_strs!(
    GRADE: "grade",
    STROKE_COUNT: "stroke_count",
    FREQ: "freq",
    JLPT: "jlpt",
);

fn parse_misc(n: Node) -> Result<Misc, ParserError> {
    let mut grade: Option<Grade> = None;
    let mut stroke_counts: Vec<i32> = Vec::new();
    let mut freq: Option<i32> = None;
    let mut old_jlpt: Option<i32> = None;

    for c in n.children() {
        let tag_name = c.tag_name().name();
        let text = util::get_node_text(c);
        match tag_name {
            GRADE => {
                grade = {
                    let i = text?.parse()?;
                    match i {
                        1..=6 => Some(Grade::Kyouiku(i)),
                        8 => Some(Grade::Jouyou),
                        9 => Some(Grade::Jinmeiyou),
                        10 => Some(Grade::JouyouVariant),
                        _ => {
                            let valids: Vec<_> = vec!["1", "2", "3", "4", "5", "6", "8", "9", "10"];
                            return Err(InvalidEnumError::new(&i.to_string(), &valids).into());
                        }
                    }
                }
            }
            STROKE_COUNT => stroke_counts.push(text?.parse()?),
            FREQ => freq = Some(text?.parse()?),
            JLPT => old_jlpt = Some(text?.parse()?),
            _ => {}
        }
    }

    let stroke_count = stroke_counts
        .first()
        .ok_or(ParserError::MissingTag(STROKE_COUNT.to_owned()))?
        .to_owned();
    let stroke_miscounts: Vec<_> = stroke_counts[1..].to_vec();

    Ok(Misc {
        grade,
        stroke_count,
        stroke_miscounts,
        freq,
        old_jlpt,
    })
}

const_strs!(
    DIC_REF: "dic_ref",
    DIC_REF_TYPE: "dr_type",

    MORO_VOL: "m_vol",
    MORO_PAGE: "m_page"
);

fn parse_dic_ref_group(n: Node) -> Result<Vec<DicRef>, ParserError> {
    n.children()
        .filter(|c| c.tag_name().name() == DIC_REF)
        .map(|c| parse_dic_ref(c))
        .collect()
}

fn parse_dic_ref(n: Node) -> Result<DicRef, ParserError> {
    let num = util::get_node_text(n)?.into_owned();
    let typ_attr = util::get_node_attr(n, DIC_REF_TYPE)?;
    let typ = typ_attr.as_ref();
    let dic_ref = match typ {
        "nelson_c" => DicRef::NelsonC(num),
        "nelson_n" => DicRef::NelsonN(num),
        "halpern_njecd" => DicRef::HalpernNJECD(num),
        "halpern_kkd" => DicRef::HalpernKKD(num),
        "halpern_kkld" => DicRef::HalpernKKLD(num),
        "halpern_kkld_2ed" => DicRef::HalpernKKLD2(num),
        "heisig" => DicRef::Heisig(num),
        "heisig6" => DicRef::Heisig6(num),
        "gakken" => DicRef::Gakken(num),
        "oneill_names" => DicRef::OneillNames(num),
        "oneill_kk" => DicRef::OneillKK(num),
        "henshall" => DicRef::Henshall(num),
        "henshall3" => DicRef::Henshall3(num),
        "sh_kk" => DicRef::SHKK(num),
        "sh_kk2" => DicRef::SHKK2(num),
        "sakade" => DicRef::Sakade(num),
        "jf_cards" => DicRef::JFCards(num),
        "tutt_cards" => DicRef::TuttCards(num),
        "crowley" => DicRef::Crowley(num),
        "kanji_in_context" => DicRef::InContext(num),
        "busy_people" => DicRef::BusyPeople(num),
        "kodansha_compact" => DicRef::KodanshaCompact(num),
        "maniette" => DicRef::Maniette(num),
        "moro" => {
            let vol = match n.attribute(MORO_VOL) {
                Some(v) => Some(v.parse()?),
                None => None,
            };
            let page = match n.attribute(MORO_PAGE) {
                Some(p) => Some(p.parse()?),
                None => None,
            };

            DicRef::Moro(num, vol, page)
        }
        _ => {
            let valids = &vec![
                "nelson_c",
                "nelson_c",
                "halpern_njecd",
                "halpern_kkd",
                "halpern_kkld",
                "halpern_kkld_2ed",
                "heisig",
                "heisig6",
                "gakken",
                "oneill_names",
                "oneill_kk",
                "henshall",
                "henshall3",
                "sh_kk",
                "sh_kk2",
                "sakade",
                "jf_cards",
                "tutt_cards",
                "crowley",
                "kanji_in_context",
                "busy_people",
                "kodansha_compact",
                "maniette",
                "moro",
            ][..];
            return Err(InvalidEnumError::new(typ, valids).into());
        }
    };

    Ok(dic_ref)
}

const_strs!(
    READING_MEANING: "rmgroup",
    READING: "reading",
    READING_TYPE: "r_type",
    READING_ONYOMI_TYPE: "on_type",
    READING_JA_STATUS: "r_status",
    MEANING: "meaning",
    MEANING_LANG: "m_lang",

    NANORI: "nanori"
);

fn parse_reading_meanings(n: Node) -> Result<(Vec<ReadingMeaning>, Vec<String>), ParserError> {
    let mut reading_meanings = Vec::new();
    let mut nanori_readings = Vec::new();

    for c in n.children() {
        let tag_name = c.tag_name().name();
        match tag_name {
            READING_MEANING => {
                let rmgroup = parse_reading_group(c)?;
                reading_meanings.push(rmgroup);
            }
            NANORI => {
                let text = util::get_node_text(c)?.into_owned();
                nanori_readings.push(text);
            }
            _ => {}
        }
    }

    Ok((reading_meanings, nanori_readings))
}

fn parse_reading_group(n: Node) -> Result<ReadingMeaning, ParserError> {
    let mut readings = Vec::new();
    let mut meanings = Vec::new();

    for c in n.children() {
        let tag_name = c.tag_name().name();
        match tag_name {
            READING => {
                let reading = parse_reading(c)?;
                readings.push(reading);
            }
            MEANING => {
                let language = c.attribute(MEANING_LANG).unwrap_or("en").to_owned();
                let content = util::get_node_text(c)?.into_owned();
                meanings.push(Meaning { content, language });
            }
            _ => {}
        }
    }

    Ok(ReadingMeaning { readings, meanings })
}

fn parse_reading(n: Node) -> Result<Reading, ParserError> {
    let value = util::get_node_text(n)?.into_owned();
    let typ_attr = util::get_node_attr(n, READING_TYPE)?;
    let typ = match typ_attr.as_ref() {
        "pinyin" => ReadingType::Pinyin,
        "korean_r" => ReadingType::KoreanR,
        "korean_h" => ReadingType::KoreanH,
        "vietnam" => ReadingType::Vietnam,
        "ja_on" => {
            let jouyou_approved = get_jouyou_approved(n);
            let onyomi_typ = match util::get_node_attr(n, READING_ONYOMI_TYPE) {
                Ok(ty) => match ty.as_ref() {
                    "kan" => OnyomiType::Kan,
                    "go" => OnyomiType::Go,
                    "tou" => OnyomiType::Tou,
                    "kan'you" => OnyomiType::Kanyou,
                    _ => {
                        let valids = &["kan", "go", "tou", "kan'you"];
                        return Err(InvalidEnumError::new(ty.as_ref(), valids).into());
                    }
                },
                Err(_) => OnyomiType::None,
            };
            ReadingType::Onyomi(jouyou_approved, onyomi_typ)
        }
        "ja_kun" => {
            let jouyou_approved = get_jouyou_approved(n);
            ReadingType::Kunyomi(jouyou_approved)
        }
        _ => {
            let valids = &[
                "pinyin", "korean_r", "korean_h", "vietnam", "ja_on", "ja_kun",
            ];
            return Err(InvalidEnumError::new(typ_attr.as_ref(), valids).into());
        }
    };

    Ok(Reading { value, typ })
}

fn get_jouyou_approved(n: Node) -> bool {
    match util::get_node_attr(n, READING_JA_STATUS) {
        Ok(_) => true,
        Err(_) => false,
    }
}
