use std::sync::Arc;

use tokio::sync::Mutex;

use crate::runtime::Runtime;

pub struct App {
  runtime: Arc<Runtime>,
  state: Arc<Mutex<AppState>>,
}

impl App {

  pub fn new() -> Self {
    Self {
      runtime: Arc::new(Runtime::new()),
      state: Arc::new(Mutex::new(AppState::Initialized)),
    }
  }

  pub async fn start(&mut self) {
    match *self.state.lock().await {
      AppState::Initialized => {
        *self.state.lock().await = AppState::Running;
        
        // TODO
      },
      AppState::Running => {
        panic!("app is already running");
      },
      AppState::Stopping |
      AppState::Stopped => {
        panic!("app is already stopped");
      },
    };
  }

  pub async fn stop(&mut self) {
    match *self.state.lock().await {
      AppState::Running => {
        *self.state.lock().await = AppState::Stopping;
        self.runtime.stop().await;
        *self.state.lock().await = AppState::Stopped;
      },
      AppState::Initialized => {
        panic!("app was never started");
      },
      AppState::Stopping |
      AppState::Stopped => {
        panic!("app is already stopped");
      },
    };
  }

}

pub enum AppState {
  Initialized,
  Running,
  Stopping,
  Stopped,
}