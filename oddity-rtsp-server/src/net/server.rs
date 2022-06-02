use std::sync::Arc;

use crate::runtime::Runtime;
use crate::net::handler::Handler;
use crate::net::connection_manager::ConnectionManager;

pub struct Server<H: Handler> {
  connection_manager: ConnectionManager<H>,
}

impl<H: Handler> Server<H> {

  pub async fn start(
    handler: H,
    runtime: Arc<Runtime>,
  ) -> Self {
    Self {
      connection_manager: ConnectionManager::start(handler, runtime.clone()).await,
    }
  }

  pub async fn stop(&mut self) {
    self.connection_manager.stop().await;
  }

}