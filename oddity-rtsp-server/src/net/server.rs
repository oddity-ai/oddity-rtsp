use std::sync::Arc;

use crate::runtime::Runtime;
use crate::net::handler::Handler;
use crate::net::connection_manager::ConnectionManager;

pub struct Server {
  connection_manager: ConnectionManager,
}

impl Server {

  // TODO implement

  pub async fn start(
    handler: Handler,
    runtime: Arc<Runtime>,
  ) -> Self {
    tracing::trace!("staring server"); // TODO host port ?
    Self {
      connection_manager: ConnectionManager::start(handler, runtime.clone()).await,
    }
  }

  pub async fn stop(&mut self) {
    self.connection_manager.stop().await;
  }

}