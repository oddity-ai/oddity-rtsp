use std::sync::Arc;

use crate::runtime::Runtime;
use crate::net::connection_manager::ConnectionManager;

pub struct Server {
  connection_manager: ConnectionManager,
}

impl Server {

  pub async fn start(
    runtime: Arc<Runtime>,
  ) -> Self {
    Self {
      connection_manager: ConnectionManager::start(runtime.clone()).await,
    }
  }

  pub async fn stop(&mut self) {
    self.connection_manager.stop().await;
  }

}