use std::fmt;
use std::io::ErrorKind;
use std::sync::Arc;

use futures::SinkExt;

use tokio::net;
use tokio::select;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_util::codec;

use oddity_rtsp_protocol::{
    AsServer, Codec, Error, RequestMaybeInterleaved, ResponseMaybeInterleaved,
};

use crate::net::handler::Handler;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::runtime::Runtime;

pub enum ConnectionState {
    Disconnected(ConnectionId),
    Closed(ConnectionId),
}

pub type ConnectionStateTx = mpsc::UnboundedSender<ConnectionState>;
pub type ConnectionStateRx = mpsc::UnboundedReceiver<ConnectionState>;

pub type ResponseSenderTx = mpsc::UnboundedSender<ResponseMaybeInterleaved>;
pub type ResponseSenderRx = mpsc::UnboundedReceiver<ResponseMaybeInterleaved>;

pub struct Connection {
    worker: Task,
}

impl Connection {
    pub async fn start(
        id: ConnectionId,
        inner: net::TcpStream,
        handler: Arc<Handler>,
        state_tx: ConnectionStateTx,
        runtime: &Runtime,
    ) -> Self {
        let (sender_tx, sender_rx) = mpsc::unbounded_channel();

        tracing::trace!(%id, "starting connection");
        let worker = runtime
            .task()
            .spawn(move |task_context| {
                Self::run(
                    id,
                    inner,
                    handler,
                    state_tx,
                    sender_tx,
                    sender_rx,
                    task_context,
                )
            })
            .await;
        tracing::trace!(%id, "started connection");

        Connection { worker }
    }

    pub async fn close(&mut self) {
        tracing::trace!("closing connection");
        self.worker.stop().await;
        tracing::trace!("closed connection");
    }

    async fn run(
        id: ConnectionId,
        inner: net::TcpStream,
        handler: Arc<Handler>,
        state_tx: ConnectionStateTx,
        response_tx: ResponseSenderTx,
        mut response_rx: ResponseSenderRx,
        mut task_context: TaskContext,
    ) {
        let mut disconnected = false;

        let addr = inner
            .peer_addr()
            .map(|peer_addr| peer_addr.to_string())
            .unwrap_or("?".to_string());
        let (read, write) = inner.into_split();
        let mut inbound = codec::FramedRead::new(read, Codec::<AsServer>::new());
        let mut outbound = codec::FramedWrite::new(write, Codec::<AsServer>::new());

        loop {
            select! {
              // CANCEL SAFETY: `mpsc::UnboundedReceiver::recv` is cancel safe.
              message = response_rx.recv() => {
                match message {
                  Some(message) => {
                    match outbound.send(message).await {
                      Ok(()) => {},
                      Err(Error::Io(err)) if err.kind() == ErrorKind::ConnectionReset => {
                        disconnected = true;
                        tracing::info!(%id, %addr, "connection: client disconnected (reset)");
                        break;
                      },
                      Err(err) => {
                        tracing::error!(%err, %id, %addr, "connection: failed to send message");
                        break;
                      }
                    }
                  },
                  None => {
                    break;
                  },
                }
              },
              // CANCEL SAFETY: `StreamExt:next` is always cancel safe.
              request = inbound.next() => {
                match request {
                  Some(Ok(request)) => {
                    match request {
                      RequestMaybeInterleaved::Message(request) => {
                        let response = handler.handle(&request, &response_tx).await;
                        let response = ResponseMaybeInterleaved::Message(response);
                        match outbound.send(response).await {
                          Ok(()) => {},
                          Err(Error::Io(err)) if err.kind() == ErrorKind::ConnectionReset => {
                            disconnected = true;
                            tracing::info!(%id, %addr, "connection: client disconnected (reset)");
                            break;
                          },
                          Err(err) => {
                            tracing::error!(%err, %id, %addr, "connection: failed to send response");
                            break;
                          },
                        }
                      },
                      RequestMaybeInterleaved::Interleaved { channel, .. } => {
                        tracing::debug!(%id, %addr, %channel, "ignored request with interleaved data");
                      },
                    }
                  },
                  None => {
                    disconnected = true;
                    tracing::info!(%id, %addr, "connection: client disconnected");
                    break;
                  },
                  Some(Err(Error::Io(err))) if err.kind() == ErrorKind::ConnectionReset => {
                    disconnected = true;
                    tracing::info!(%id, %addr, "connection: client disconnected (reset)");
                    break;
                  },
                  Some(Err(err)) => {
                    tracing::error!(%err, %id, %addr, "connection: failed to read request");
                    break;
                  },
                }
              },
              // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
              _ = task_context.wait_for_stop() => {
                tracing::trace!(%id, %addr, "connection worker stopping");
                break;
              },
            };
        }

        if disconnected {
            // Client disconnected.
            let _ = state_tx.send(ConnectionState::Disconnected(id));
        } else {
            // Reason for breaking out of loop was unexpected and not due to the
            // client disconnecting.
            let _ = state_tx.send(ConnectionState::Closed(id));
        }
        tracing::trace!(%id, %addr, "connection worker EOL");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ConnectionId(usize);

impl ConnectionId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct ConnectionIdGenerator(usize);

impl ConnectionIdGenerator {
    pub fn new() -> Self {
        ConnectionIdGenerator(0)
    }

    pub fn generate(&mut self) -> ConnectionId {
        let id = self.0;
        self.0 += 1;
        ConnectionId::new(id)
    }
}
