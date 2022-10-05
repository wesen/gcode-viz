use crate::App;
use std::sync::Arc;
use std::time::Duration;
use log::info;

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
                info!("Initializing the application");
                let mut app = self.app.lock().await;
                tokio::time::sleep(Duration::from_secs(1)).await;
                app.initialized();
            }
            IoEvent::Sleep(duration) => {
                tokio::time::sleep(duration).await;
            }
        }
    }
}
