pub mod config;
pub mod handler;

use std::error::Error;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::app::config::AppConfig;
use crate::app::handler::AppHandler;
use crate::net::server::Server;
use crate::runtime::Runtime;
use crate::session::session_manager::SessionManager;
use crate::source::source_manager::SourceManager;

macro_rules! handle_err {
    ($rt:ident, $expr:expr) => {
        match $expr {
            Ok(ret) => Ok(ret),
            Err(err) => {
                $rt.stop().await;
                Err(err)
            }
        }
    };
}

pub struct App {
    server: Server,
    context: Arc<RwLock<AppContext>>,
    runtime: Arc<Runtime>,
}

impl App {
    #[allow(clippy::missing_errors_doc, clippy::future_not_send)]
    pub async fn start(config: AppConfig) -> Result<Self, Box<dyn Error>> {
        let runtime = Arc::new(Runtime::new());

        let mut context = initialize_context(runtime.clone()).await;
        handle_err!(
            runtime,
            register_sources_with_context(&config, &mut context,).await
        )?;

        let context = Arc::new(RwLock::new(context));
        let server = handle_err!(
            runtime,
            initialize_server(&config, context.clone(), runtime.clone(),).await
        )?;

        Ok(Self {
            server,
            context,
            runtime,
        })
    }

    pub async fn stop(&mut self) {
        self.server.stop().await;
        self.context.write().await.session_manager.stop().await;
        self.context.write().await.source_manager.stop().await;
        self.runtime.stop().await;
    }
}

async fn initialize_server(
    config: &AppConfig,
    context: Arc<RwLock<AppContext>>,
    runtime: Arc<Runtime>,
) -> Result<Server, Box<dyn Error>> {
    let handler = AppHandler::new(context.clone());
    Server::start(
        config.server.host.parse()?,
        config.server.port,
        handler,
        runtime.clone(),
    )
    .await
    .map_err(Into::into)
}

async fn initialize_context(runtime: Arc<Runtime>) -> AppContext {
    AppContext {
        source_manager: SourceManager::start(runtime.clone()).await,
        session_manager: SessionManager::start(runtime.clone()).await,
    }
}

#[allow(
    clippy::missing_errors_doc,
    clippy::future_not_send,
    clippy::needless_pass_by_ref_mut
)]
async fn register_sources_with_context(
    config: &AppConfig,
    context: &mut AppContext,
) -> Result<(), Box<dyn Error>> {
    tracing::trace!("registering sources");
    for item in &config.media {
        tracing::info!(%item, "registering source");
        context
            .source_manager
            .register_and_start(
                item.name.as_str(),
                item.path.clone(),
                item.as_media_descriptor()?,
            )
            .await?;
    }
    tracing::trace!("registered sources");
    Ok(())
}

pub struct AppContext {
    source_manager: SourceManager,
    session_manager: SessionManager,
}
