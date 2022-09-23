use std::io::stdout;
use std::path::PathBuf;
use std::time::Duration;

use crate::app::AppReturn;
use crate::ui::{Events, InputEvent};
use app::App;
use clap::Parser;
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod actions;
mod app;
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

fn start_ui() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let app = App::new();
    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    loop {
        terminal.draw(|rect| ui::draw(rect, &app))?;

        let result = match events.next()? {
            InputEvent::Input(key) => app.do_action(key),
            InputEvent::Tick => app.update_on_tick(),
        };
        if result == AppReturn::Exit {
            break;
        }
    }

    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn main() {
    start_ui().unwrap();
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
}
