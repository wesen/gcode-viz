use crate::app::App;
use crate::io::IoEvent;
use crate::key::Key;
use crate::AppReturn;
use crossterm::event;
use std::io::stdout;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::{Frame, Terminal};

pub fn draw<B>(rect: &mut Frame<B>, _app: &App)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    let _chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3)].as_ref())
        .split(size);

    let block = draw_title();

    rect.render_widget(block, size);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Plop with TUI")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}

fn check_size(rect: &Rect) {
    if rect.width < 52 || rect.height < 28 {
        panic!("Terminal too small");
    }
}

pub enum InputEvent {
    /// An input event occured
    Input(Key),
    /// A tick event occured
    Tick,
}

/// A small event handler that wraps crossterm input and tick events.
pub struct Events {
    rx: Receiver<InputEvent>,
    // Needs to be kept around to prevent disposing the sender side.
    _tx: Sender<InputEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone();
        thread::spawn(move || {
            if event::poll(tick_rate).unwrap() {
                if let event::Event::Key(keyEvent) = event::read().unwrap() {
                    let key = Key::from(keyEvent);
                    event_tx.send(InputEvent::Input(key)).unwrap();
                }
            }
        });

        Events { rx, _tx: tx }
    }

    /// Attempts to read an event.
    /// This function blocks the current thread.
    pub fn next(&self) -> Result<InputEvent, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }
}

pub async fn start_ui(app: Arc<tokio::sync::Mutex<App>>) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    {
        let mut app = app.lock().await;
        app.dispatch(IoEvent::Initialize);
    }

    loop {
        let mut app = app.lock().await;

        terminal.draw(|rect| draw(rect, &app))?;

        // TODO(manuel) needs to be async
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
