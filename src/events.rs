use crate::key::Key;
use crate::ui::InputEvent;
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// A small event handler that wraps crossterm input and tick events.
/// Each event type is handled in its own thread and returned to a common
/// `Receiver`
pub struct Events {
    rx: tokio::sync::mpsc::Receiver<InputEvent>,
    // keep around to avoid disposing of sender side
    _tx: tokio::sync::mpsc::Sender<InputEvent>,
    // TODO(manuel) why Arc?
    stop_capture: Arc<AtomicBool>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = tokio::sync::mpsc::channel::<InputEvent>(100);
        let stop_capture = Arc::new(AtomicBool::new(false));
        let stop_capture_clone = Arc::clone(&stop_capture);
        // keep tx alive I guess, since its scope outlives the thread
        let _tx = tx.clone();

        tokio::spawn(async move {
            loop {
                if crossterm::event::poll(tick_rate).unwrap() {
                    if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                        let key = Key::from(key);
                        if let Err(err) = tx.send(InputEvent::Input(key)).await {
                            error!("Ooops!, {}", err);
                        }
                    }
                }
                if let Err(err) = tx.send(InputEvent::Tick).await {
                    error!("Ooops!, {}", err);
                }
                if stop_capture_clone.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Events {
            rx,
            _tx,
            stop_capture,
        }
    }

    pub async fn next(&mut self) -> InputEvent {
        self.rx.recv().await.unwrap_or(InputEvent::Tick)
    }

    pub fn close(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed);
    }
}
