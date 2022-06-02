pub mod handler;

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::runtime::Runtime;
use crate::net::server::Server;
use crate::source::source_manager::SourceManager;
use crate::session::session_manager::SessionManager;
use crate::app::handler::AppHandler;

#[derive(Clone)]
pub enum AppState {
  Running,
  Stopping,
  Stopped,
}

pub struct App {
  server: Server,
  context: Arc<AppContext>,
  runtime: Arc<Runtime>,
  state: Arc<Mutex<AppState>>,
}

impl App {

  pub async fn start() -> App {
    let runtime = Arc::new(Runtime::new());
    let context = Arc::new(AppContext {
      source_manager: SourceManager::start(runtime.clone()).await,
      session_manager: SessionManager::start(runtime.clone()).await,
    });
    let handler = AppHandler::new(context.clone());
    Self {
      server: Server::start(handler, runtime.clone()).await,
      context,
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

pub struct AppContext {
  source_manager: SourceManager,
  session_manager: SessionManager,
}