use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use serde_either::SingleOrVec;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct ParameterValue {
    #[serde(flatten)]
    pub tag: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Parameter {
    pub tag: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(flatten)]
    pub since: Option<String>,
    pub description: Option<String>,
    pub requires: Option<String>,
    pub values: Option<Vec<ParameterValue>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Example {
    pub pre: Option<SingleOrVec<String>>,
    pub code: SingleOrVec<String>,
    pub post: Option<SingleOrVec<String>>,
}

#[derive(Clone, Deserialize, Debug)]
#[allow(unused)]
pub struct OpcodeDescription {
    pub tag: String,
    pub title: String,
    pub brief: String,
    pub author: Option<String>,

    pub experimental: Option<bool>,
    #[serde(flatten)]
    pub since: Option<String>,
    pub requires: Option<String>,

    pub parameters: Option<SingleOrVec<Parameter>>,

    pub videos: Option<Vec<String>>,

    pub group: Option<SingleOrVec<String>>,
    pub codes: Vec<String>,
    pub notes: Option<SingleOrVec<String>>,
    pub examples: Option<SingleOrVec<Example>>,
}

pub fn parse_marlin_docs(gcode_docs_dir: String) -> HashMap<String, Rc<OpcodeDescription>> {
    let opcodes: HashMap<String, Rc<OpcodeDescription>> = std::fs::read_dir(gcode_docs_dir)
        .unwrap()
        // .take(5)
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().unwrap().is_file() && entry.path().extension().unwrap() == "md"
        })
        .map(|entry| entry.path())
        .map(parse_opcode_md)
        .filter_map(|opcode| match opcode {
            Ok(opcode) => Some(opcode),
            Err(err) => {
                println!("Error parsing opcode: {}", err);
                None
            }
        })
        .flatten()
        .collect();
    opcodes
}

pub fn parse_opcode_md(
    doc: PathBuf,
) -> Result<Vec<(String, Rc<OpcodeDescription>)>, Box<dyn std::error::Error>> {
    let matter = Matter::<YAML>::new();
    let s = std::fs::read_to_string(doc.clone())?;
    let result = matter.parse(s.as_str());

    let od: OpcodeDescription = result.data.unwrap().deserialize()?;
    let od = Rc::new(od);

    let _parser = pulldown_cmark::Parser::new(result.content.as_str());
    // for event in parser {
    //     println!("{:?}", event);
    // }
    let codes: Vec<(String, Rc<OpcodeDescription>)> = od
        .codes
        .iter()
        .map(|code| (code.clone(), od.clone()))
        .collect();

    Ok(codes)
}
