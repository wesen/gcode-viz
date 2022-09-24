use crate::app::{App, AppReturn};
use crate::io::{IoAsyncHandler, IoEvent};
use clap::Parser;
use eyre::Result;
use std::path::PathBuf;
use std::sync::Arc;

mod actions;
mod app;
mod io;
mod key;
mod marlin_docs;
mod ui;
mod events;

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

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));

    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    // TODO(manuel) needs to be async
    ui::start_ui(&app).await?;

    let args = Cli::parse();

    // append /_gcode to the marlin docs dir
    let gcode_docs_dir = format!("{}/_gcode", args.marlin_docs_dir);
    // get all files in doc_dir
    let opcodes = marlin_docs::parse_marlin_docs(gcode_docs_dir);

    let path = args.file.clone();
    let s = std::fs::read_to_string(path).unwrap();
    if args.file.ends_with(".gcode") {
        println!("Parsing GCode file: {}", args.file);
        for i in gcode::parse(s.as_str()) {
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

    Ok(())
}
