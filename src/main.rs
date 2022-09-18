use std::path::PathBuf;

use clap::Parser;

mod marlin_docs;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Name of the GCode file to parse
    #[clap(value_parser)]
    file: String,

    /// Name of the directory with marlin documentation
    #[clap(
        short,
        long,
        value_parser,
        default_value = "vendor/MarlinDocumentation"
    )]
    marlin_docs_dir: String,
}

fn main() {
    let args = Cli::parse();

    // append /_gcode to the marlin docs dir
    let gcode_docs_dir = format!("{}/_gcode", args.marlin_docs_dir);
    // get all files in doc_dir
    let opcodes = marlin_docs::parse_marlin_docs(gcode_docs_dir);

    let path = args.file.clone();
    let s = std::fs::read_to_string(path).unwrap();
    if args.file.ends_with(".gcode") {
        println!("Parsing GCode file: {}", args.file);
        for i in gcode::parse(s.as_str()).take(50) {
            let opcode = match (i.mnemonic(), i.major_number(), i.minor_number()) {
                (m, major, 0) => format!("{}{}", m, major),
                (m, major, minor) => format!("{}{}.{}", m, major, minor),
            };
            let span = i.span();
            let orig = s[span.start..span.end].to_string();
            if let Some(od) = opcodes.get(&opcode) {
                println!("{}: {}", opcode, od.title);
            } else {
                println!("{}: {}", opcode, "Unknown");
            }
            println!("{:?} - {}", opcode, orig);
        }
    } else if args.file.ends_with(".md") {
        let od = marlin_docs::parse_opcode_md(PathBuf::from(args.file)).unwrap();
        println!("{:?}", od);
    } else {
        println!("File is not a GCode file");
    }
}
