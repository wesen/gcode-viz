use clap::Parser;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use serde_either::SingleOrVec;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Name of the GCode file to parse
    #[clap(value_parser)]
    file: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct ParameterValue {
    #[serde(flatten)]
    tag: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Parameter {
    tag: String,
    #[serde(default)]
    optional: bool,
    #[serde(flatten)]
    since: Option<String>,
    description: Option<String>,
    values: Option<Vec<ParameterValue>>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Example {
    pre: Option<SingleOrVec<String>>,
    code: SingleOrVec<String>,
    post: Option<SingleOrVec<String>>,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
struct OpcodeDescription {
    tag: String,
    title: String,
    brief: String,
    author: Option<String>,

    experimental: Option<bool>,
    #[serde(flatten)]
    since: Option<String>,
    requires: Option<String>,

    parameters: Option<SingleOrVec<Parameter>>,

    videos: Option<Vec<String>>,

    group: Option<SingleOrVec<String>>,
    codes: Vec<String>,
    notes: Option<SingleOrVec<String>>,
    examples: Option<SingleOrVec<Example>>,
}

fn main() {
    let args = Cli::parse();

    let path = args.file.clone();
    let s = std::fs::read_to_string(path).unwrap();
    if args.file.ends_with(".gcode") {
        println!("Parsing GCode file: {}", args.file);
        for i in gcode::parse(s.as_str()).take(10) {
            println!("{:?}", i);
        }
    } else if args.file.ends_with(".md") {
        let matter = Matter::<YAML>::new();
        let result = matter.parse(s.as_str());

        let od: OpcodeDescription = result.data.unwrap().deserialize().unwrap();
        println!("{:?}", od);

        let _parser = pulldown_cmark::Parser::new(result.content.as_str());
        // for event in parser {
        //     println!("{:?}", event);
        // }
    } else {
        println!("File is not a GCode file");
    }
}
