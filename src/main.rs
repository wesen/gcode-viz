use crate::ui::app::App;
use crate::ui::io::{IoAsyncHandler, IoEvent};
use clap::Parser;
use eyre::Result;
use gcode::{Callbacks, Comment, Nop};
use gcode_viz::gcode::lines::{DisplayLine, LineIterator};
use gcode_viz::gcode::marlin_docs;
use gcode_viz::helpers::PopIf;
use std::path::PathBuf;
use std::sync::Arc;

mod ui;

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

async fn run_ui() -> Result<(), eyre::Error> {
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_clone = Arc::clone(&app);

    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    ui::start_ui(app_clone).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    color_eyre::install()?;

    let args = Cli::parse();

    // append /_gcode to the marlin docs dir
    let gcode_docs_dir = format!("{}/_gcode", args.marlin_docs_dir);
    // get all files in doc_dir
    let opcodes = marlin_docs::parse_marlin_docs(gcode_docs_dir);

    let path = args.file.clone();
    let s = std::fs::read_to_string(path).unwrap();
    if args.file.ends_with(".gcode") {
        println!("Parsing GCode file: {}", args.file);
        let lines = gcode::full_parse_with_callbacks(s.as_str(), Nop);

        let mut my_iterator = LineIterator::new(lines.take(32));
        my_iterator.for_each(|line| match line {
            DisplayLine::Comment(c) => println!("// {}", c.value),
            DisplayLine::GCode(o, opcode) => {
                let span = opcode.span();
                let orig = s[span.start..span.end].to_string();
                if let Some(od) = opcodes.get(o.as_str()) {
                    println!("{}: {}", opcode, od.title);
                } else {
                    println!("{}: {}", opcode, "Unknown");
                }
            }
        });
    } else if args.file.ends_with(".md") {
        let od = marlin_docs::parse_opcode_md(PathBuf::from(args.file)).unwrap();
        println!("{:?}", od);
    } else {
        println!("File is not a GCode file");
    }

    Ok(())
}
