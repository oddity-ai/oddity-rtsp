pub mod config;
pub mod handler;

use std::error::Error;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::runtime::Runtime;
use crate::net::server::Server;
use crate::source::source_manager::SourceManager;
use crate::session::session_manager::SessionManager;
use crate::app::handler::AppHandler;
use crate::app::config::AppConfig;

#[derive(Clone)]
pub enum AppState {
  Running,
  Stopping,
  Stopped,
}

pub struct App {
  server: Server,
  state: Arc<Mutex<AppState>>,
  context: Arc<Mutex<AppContext>>,
  runtime: Arc<Runtime>,
}

impl App {

  pub async fn start(config: AppConfig) -> Result<App, Box<dyn Error>> {
    let runtime = Arc::new(Runtime::new());
    let mut context = AppContext {
      source_manager: SourceManager::start(runtime.clone()).await,
      session_manager: SessionManager::start(runtime.clone()).await,
    };
    tracing::trace!("registering sources");
    for item in config.media {
      tracing::info!(%item, "registering source");
      context
        .source_manager
        .register_and_start(
          item.name.as_str(),
          item.path.clone(),
          item.as_media_descriptor()?,
        ).await?;
    }
    tracing::trace!("registered sources");

    let context = Arc::new(Mutex::new(context));
    let handler = AppHandler::new(context.clone());
    Ok(Self {
      server: Server::start(
        config.server.host.parse()?,
        config.server.port,
        handler,
        runtime.clone(),
      ).await?,
      state: Arc::new(Mutex::new(AppState::Running)),
      context,
      runtime,
    })
  }

  pub async fn stop(&mut self) {
    match *self.state.lock().await {
      AppState::Running => {
        self.server.stop().await;
        *self.state.lock().await = AppState::Stopping;
        {
          let mut context = self.context.lock().await;
          context.source_manager.stop().await;
          context.session_manager.stop().await;
        }
        self.runtime.stop().await;
        *self.state.lock().await = AppState::Stopped;
      },
      AppState::Stopping |
      AppState::Stopped => {
        panic!("app is already stopped");
      },
    };
  }

}

pub struct AppContext {
  source_manager: SourceManager,
  session_manager: SessionManager,
}