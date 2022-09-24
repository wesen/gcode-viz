use crate::actions::{Action, Actions};
use crate::io::IoEvent;
use crate::key::Key;
use std::time::Duration;

#[derive(Clone)]
pub enum AppState {
    Init,
    Initialized {
        duration: Duration,
        counter_sleep: u32,
        counter_tick: u64,
    },
}

impl AppState {
    pub fn initialized() -> Self {
        let duration = Duration::from_secs(1);
        let counter_sleep = 0;
        let counter_tick = 0;
        Self::Initialized {
            duration,
            counter_sleep,
            counter_tick,
        }
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized { .. })
    }

    pub fn incr_sleep(&mut self) {
        if let Self::Initialized { counter_sleep, .. } = self {
            *counter_sleep += 1;
        }
    }

    pub fn incr_tick(&mut self) {
        if let Self::Initialized { counter_tick, .. } = self {
            *counter_tick += 1;
        }
    }

    pub fn count_sleep(&self) -> Option<u32> {
        if let Self::Initialized { counter_sleep, .. } = self {
            Some(*counter_sleep)
        } else {
            None
        }
    }

    pub fn count_tick(&self) -> Option<u64> {
        if let Self::Initialized { counter_tick, .. } = self {
            Some(*counter_tick)
        } else {
            None
        }
    }

    pub fn duration(&self) -> Option<Duration> {
        if let Self::Initialized { duration, .. } = self {
            Some(*duration)
        } else {
            None
        }
    }

    pub fn increment_delay(&mut self) {
        if let Self::Initialized { duration, .. } = self {
            // clamp duration to 1 - 10
            let secs = (duration.as_secs() + 1).clamp(1, 10);
            *duration = Duration::from_secs(secs);
        }
    }

    pub fn decrement_delay(&mut self) {
        if let Self::Initialized { duration, .. } = self {
            // clamp duration to 1 - 10
            let secs = (duration.as_secs() - 1).clamp(1, 10);
            *duration = Duration::from_secs(secs);
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}

#[allow(unused)]
pub struct App {
    /// Contextual actions
    actions: Actions,
    /// State
    state: AppState,
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Continue,
    Exit,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        Self {
            actions: vec![Action::Quit].into(),
            state: AppState::default(),
            io_tx,
        }
    }

    pub fn do_action(&self, key: Key) -> AppReturn {
        if key == Key::Esc {
            return AppReturn::Exit;
        }
        AppReturn::Continue
    }

    pub fn dispatch(&self, event: IoEvent) {
        match event {
            IoEvent::Sleep(duration) => if self.state.is_initialized() {},
            IoEvent::Initialize => {}
        }
        todo!()
    }

    pub fn update_on_tick(&self) -> AppReturn {
        AppReturn::Continue
    }
}
