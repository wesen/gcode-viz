use crate::app::{App, AppReturn};
use crate::io::{IoAsyncHandler, IoEvent};
use clap::Parser;
use eyre::Result;
use gcode::{Callbacks, Comment, Nop};
use gcode_viz::helpers::PopIf;
use std::path::PathBuf;
use std::sync::Arc;

mod actions;
mod app;
mod events;
mod io;
mod key;
mod marlin_docs;
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

struct MyCallbacks {}

impl Callbacks for &MyCallbacks {}

enum DisplayLine<'a> {
    Comment(gcode::Comment<'a>),
    GCode(String, gcode::GCode),
}

struct LineIterator<'input, I>
where
    I: Iterator<Item = gcode::Line<'input>>,
{
    s: I,
    current_line: Option<gcode::Line<'input>>,
    comments: Vec<gcode::Comment<'input>>,
    gcodes: Vec<gcode::GCode>,
}

impl<'input, I> LineIterator<'input, I>
where
    I: Iterator<Item = gcode::Line<'input>>,
{
    fn new(mut lines: I) -> Self {
        let mut res = LineIterator {
            s: lines,
            current_line: None,
            comments: Vec::new(),
            gcodes: Vec::new(),
        };
        res.next_line();

        res
    }

    fn next_line(&mut self) {
        self.current_line = self.s.next();
        if let Some(s) = &self.current_line {
            self.comments.extend(s.comments().iter().cloned());
            self.gcodes.extend(s.gcodes().iter().cloned());
        }
    }
}

impl<'input, I> Iterator for LineIterator<'input, I>
where
    I: Iterator<Item = gcode::Line<'input>> + 'input,
{
    type Item = DisplayLine<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(l) = &self.current_line {
            let first_gcode_line: usize = self
                .gcodes
                .get(0)
                .and_then(|x| Some(x.span().line))
                .unwrap_or(0);

            // if there are still comments for previous lines, emit
            if let Some(x) = self.comments.pop_if(|c| c.span.line <= first_gcode_line) {
                return Some(DisplayLine::Comment(x.clone()));
            }

            // emit all gcodes buffered up
            if let Some(i) = self.gcodes.pop() {
                let opcode = match (i.mnemonic(), i.major_number(), i.minor_number()) {
                    (m, major, 0) => format!("{}{}", m, major),
                    (m, major, minor) => format!("{}{}.{}", m, major, minor),
                };
                return Some(DisplayLine::GCode(opcode, i.clone()));
            }

            // all gcodes emitted, get next line
            self.next_line();
        }

        None
    }
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
        let mut display_lines: Vec<DisplayLine> = Vec::new();

        let mut myIterator = LineIterator::new(lines.take(32));
        myIterator.for_each(|line| match line {
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
