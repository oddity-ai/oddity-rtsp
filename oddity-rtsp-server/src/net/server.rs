use std::net::IpAddr;
use std::sync::Arc;

use tokio::net;
use tokio::select;

use crate::net::connection_manager::ConnectionManager;
use crate::net::handler::Handler;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::runtime::Runtime;

type Result<T> = std::result::Result<T, std::io::Error>;

pub struct Server {
    worker: Task,
}

impl Server {
    pub async fn start(
        host: IpAddr,
        port: u16,
        handler: Handler,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        tracing::trace!(%host, port, "starting server");
        let listener = match net::TcpListener::bind((host, port)).await {
            Ok(listener) => listener,
            Err(err) => {
                tracing::error!(%err, %host, port, "failed to listen for connections");
                return Err(err);
            }
        };
        tracing::info!(%host, port, "server listening for incoming connections");

        let worker = runtime
            .task()
            .spawn({
                let runtime = runtime.clone();
                move |task_context| Self::run(listener, handler, runtime, task_context)
            })
            .await;
        tracing::trace!(%host, port, "started server");

        Ok(Self { worker })
    }

    pub async fn stop(&mut self) {
        tracing::trace!("sending stop signal to server");
        self.worker.stop().await;
        tracing::trace!("server stopped");
    }

    async fn run(
        listener: net::TcpListener,
        handler: Handler,
        runtime: Arc<Runtime>,
        mut task_context: TaskContext,
    ) {
        let mut connection_manager = ConnectionManager::start(handler, runtime).await;
        loop {
            select! {
              // CANCEL SAFETY: `tokio::net::TcpListener::accept` is cancel safe.
              incoming = listener.accept() => {
                match incoming {
                  Ok((incoming, peer_addr)) => {
                    tracing::trace!(%peer_addr, "accepted client");
                    connection_manager.spawn(incoming).await;
                  },
                  Err(err) => {
                    tracing::error!(%err, "failed to accept connection");
                  },
                }
              },
              // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
              _ = task_context.wait_for_stop() => {
                tracing::trace!("server stopping");
                break;
              },
            }
        }

        connection_manager.stop().await;
    }
}
