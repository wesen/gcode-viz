use crate::App;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,
    Sleep(Duration),
}

pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        match io_event {
            IoEvent::Initialize => {
                self.app.lock().await.init();
            }
            IoEvent::Sleep(duration) => {
                tokio::time::sleep(duration).await;
            }
        }
    }
}
