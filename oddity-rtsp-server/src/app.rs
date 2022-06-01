use std::sync::Arc;

use tokio::sync::Mutex;

use crate::runtime::Runtime;
use crate::net::server::Server;
use crate::source::source_manager::SourceManager;
use crate::session::session_manager::SessionManager;

pub struct App {
  server: Server,
  source_manager: SourceManager,
  session_manager: SessionManager,
  runtime: Arc<Runtime>,
  state: Arc<Mutex<AppState>>,
}

impl App {

  pub async fn start() -> App {
    let runtime = Arc::new(Runtime::new());
    Self {
      server: Server::start(runtime.clone()).await,
      source_manager: SourceManager::start(runtime.clone()).await,
      session_manager: SessionManager::start(runtime.clone()).await,
      runtime,
      state: Arc::new(Mutex::new(AppState::Running)),
    }
  }

  pub async fn stop(&mut self) {
    match *self.state.lock().await {
      AppState::Running => {
        *self.state.lock().await = AppState::Stopping;
        self.runtime.stop().await;
        *self.state.lock().await = AppState::Stopped;
      },
      AppState::Stopping |
      AppState::Stopped => {
        panic!("app is already stopped");
      },
    };
  }

  pub async fn state(&self) -> AppState {
    self.state.lock().await.clone()
  }

}

#[derive(Clone)]
pub enum AppState {
  Running,
  Stopping,
  Stopped,
}