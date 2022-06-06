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

pub struct App {
  server: Server,
  context: Arc<Mutex<AppContext>>,
  runtime: Arc<Runtime>,
}

impl App {

  // TODO failure can occur within app even when some contexts are
  // already started which will cause them to break due to channels
  // being dropped left and right, how can we handle this gracefully?
  // idea: split start into runtime only and rest, if rest fails, stop
  // runtime before handing back error!!
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
      context,
      runtime,
    })
  }

  pub async fn stop(&mut self) {
    self.server.stop().await;
    {
      let mut context = self.context.lock().await;
      context.source_manager.stop().await;
      context.session_manager.stop().await;
    }
    self.runtime.stop().await;
  }

}

pub struct AppContext {
  source_manager: SourceManager,
  session_manager: SessionManager,
}