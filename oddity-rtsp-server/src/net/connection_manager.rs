use std::collections::HashMap;
use std::sync::Arc;

use tokio::net;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use crate::net::connection::{
    Connection, ConnectionId, ConnectionIdGenerator, ConnectionState, ConnectionStateRx,
    ConnectionStateTx,
};
use crate::net::handler::Handler;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::runtime::Runtime;

type ConnectionMap = Arc<Mutex<HashMap<ConnectionId, Connection>>>;

pub struct ConnectionManager {
    connections: ConnectionMap,
    connection_id_generator: ConnectionIdGenerator,
    connection_state_tx: ConnectionStateTx,
    handler: Arc<Handler>,
    worker: Task,
    runtime: Arc<Runtime>,
}

impl ConnectionManager {
    pub async fn start(handler: Handler, runtime: Arc<Runtime>) -> Self {
        let connections = Arc::new(Mutex::new(HashMap::new()));

        let (connection_state_tx, connection_state_rx) = mpsc::unbounded_channel();

        tracing::trace!("starting connection manager");
        let worker = runtime
            .task()
            .spawn({
                let connections = connections.clone();
                |task_context| Self::run(connections, connection_state_rx, task_context)
            })
            .await;
        tracing::trace!("started connection manager");

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
        tracing::trace!("sending stop signal to connection manager");
        self.worker.stop().await;
        tracing::trace!("connection manager stopped");
        for (_, mut connection) in self.connections.lock().await.drain() {
            connection.close().await;
        }
    }

    pub async fn spawn(&mut self, stream: net::TcpStream) {
        let id = self.connection_id_generator.generate();
        let connection = Connection::start(
            id,
            stream,
            self.handler.clone(),
            self.connection_state_tx.clone(),
            self.runtime.as_ref(),
        )
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
              // CANCEL SAFETY: `mpsc::UnboundedReceiver::recv` is cancel safe.
              connection_state = connection_state_rx.recv() => {
                match connection_state {
                  Some(ConnectionState::Disconnected(connection_id)) => {
                    tracing::trace!(
                      %connection_id,
                      "connection manager: received disconnected",
                    );
                    connections.lock().await.remove(&connection_id);
                  },
                  Some(ConnectionState::Closed(connection_id)) => {
                    tracing::trace!(
                      %connection_id,
                      "connection manager: received closed",
                    );
                    connections.lock().await.remove(&connection_id);
                  },
                  None => {
                    tracing::error!("connection state channel broke unexpectedly");
                    break;
                  },
                }
              },
              // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
              _ = task_context.wait_for_stop() => {
                tracing::trace!("connection manager worker stopping");
                break;
              },
            }
        }
    }
}
