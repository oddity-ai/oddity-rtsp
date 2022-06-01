use std::sync::Arc;

use crate::runtime::Runtime;
use crate::net::connection_manager::ConnectionManager;

pub struct Server {
  connection_manager: ConnectionManager,
}

impl Server {

  pub async fn run(
    runtime: Arc<Runtime>,
  ) -> Self {
    Self {
      connection_manager: ConnectionManager::new(runtime.clone()),
    }
  }

}