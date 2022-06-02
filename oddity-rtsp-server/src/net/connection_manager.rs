use std::sync::Arc;
use std::collections::HashMap;

use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::net;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::net::handler::Handler;
use crate::net::connection::{
  Connection,
  ConnectionId,
  ConnectionIdGenerator,
  ConnectionState,
  ConnectionStateTx,
  ConnectionStateRx,
};

type ConnectionMap = Arc<Mutex<HashMap<ConnectionId, Connection>>>;

pub struct ConnectionManager<H: Handler> {
  connections: ConnectionMap,
  connection_id_generator: ConnectionIdGenerator,
  connection_state_tx: ConnectionStateTx,
  handler: Arc<H>,
  worker: Task,
  runtime: Arc<Runtime>,
}

impl<H: Handler> ConnectionManager<H> {

  pub async fn start(
    handler: H,
    runtime: Arc<Runtime>,
  ) -> Self {
    let connections = Arc::new(Mutex::new(HashMap::new()));

    let (connection_state_tx, connection_state_rx)
      = mpsc::unbounded_channel();

    let worker = runtime
      .task()
      .spawn({
        let connections = connections.clone();
        |task_context| {
          Self::run(
            connections,
            connection_state_rx,
            task_context,
          )
        }
      })
      .await;

    Self {
      connections,
      connection_id_generator: ConnectionIdGenerator::new(),
      connection_state_tx,
      handler: Arc::new(handler),
      worker,
      runtime,
    }
  }

  pub async fn stop(&mut self) {
    self.worker.stop().await;
    // TODO move this into run???
    for (_, mut connection) in self.connections.lock().await.drain() {
      connection.close().await;
    }
  }

  pub async fn spawn(
    &mut self,
    stream: net::TcpStream,
  ) {
    let id = self.connection_id_generator.generate();
    let connection = Connection::start(
        id,
        stream,
        self.handler.clone(),
        self.connection_state_tx.clone(),
        self.runtime.as_ref())
      .await;

    self.connections.lock().await.insert(id, connection);
  }

  async fn run(
    connections: ConnectionMap,
    mut connection_state_rx: ConnectionStateRx,
    mut task_context: TaskContext,
  ) {
    loop {
      select! {
        connection_state = connection_state_rx.recv() => {
          match connection_state {
            Some(ConnectionState::Disconnected(connection_id)) => {
              connections.lock().await.remove(&connection_id);
            },
            Some(ConnectionState::Closed(connection_id)) => {
              connections.lock().await.remove(&connection_id);
            },
            None => {
              break;
            },
          }
        },
        _ = task_context.wait_for_stop() => {
          break;
        },
      }
    }
  }

}