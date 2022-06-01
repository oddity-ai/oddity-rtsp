use futures::SinkExt;
use tokio::select;
use tokio::sync::mpsc;
use tokio::net;
use tokio_stream::StreamExt;
use tokio_util::codec;

use rand::random;

use oddity_rtsp_protocol::{Codec, AsServer, ResponseMaybeInterleaved};

use crate::runtime::Runtime;
use crate::runtime::task_manager::TaskContext;

pub enum ConnectionState {
  Disconnected(ConnectionId),
  Closed(ConnectionId),
}

pub type ConnectionStateTx = mpsc::UnboundedSender<ConnectionState>;
pub type ConnectionStateRx = mpsc::UnboundedReceiver<ConnectionState>;

pub type ResponseSenderTx = mpsc::UnboundedSender<ResponseMaybeInterleaved>;
pub type ResponseSenderRx = mpsc::UnboundedReceiver<ResponseMaybeInterleaved>;

pub struct Connection {
  sender_tx: ResponseSenderTx,
}

impl Connection {

  pub async fn start(
    id: ConnectionId,
    inner: net::TcpStream,
    state_tx: ConnectionStateTx,
    runtime: &Runtime,
  ) -> Self {
    let (sender_tx, sender_rx) = mpsc::unbounded_channel();

    runtime
      .task()
      .spawn(
        move |task_context| Self::run(
          id,
          inner,
          state_tx,
          sender_rx,
          task_context,
        )
      )
      .await;

    Connection {
      sender_tx,
    }
  }

  pub fn sender_tx(&self) -> ResponseSenderTx {
    self.sender_tx.clone()
  }

  async fn run(
    id: ConnectionId,
    inner: net::TcpStream,
    state_tx: ConnectionStateTx,
    mut response_rx: ResponseSenderRx,
    mut task_context: TaskContext,
  ) {
    let (read, write) = inner.into_split();
    let mut inbound = codec::FramedRead::new(read, Codec::<AsServer>::new());
    let mut outbound = codec::FramedWrite::new(write, Codec::<AsServer>::new());

    loop {
      select! {
        packet = response_rx.recv() => {
          match packet {
            Some(packet) => {
              if let Err(err) = outbound.send(packet).await {
                // TODO handle
                // TODO this is a complicated piece of the puzzle because we need to figure
                // out how connection communicates with the source and session manager with-
                // out too much hackiness... channels????
              }
            },
            None => {
              break;
            },
          }
        },
        packet = inbound.next() => {
          match packet {
            Some(Ok(packet)) => {
              // TODO
            },
            Some(Err(err)) => {
              // TODO
            },
            None => {
              let _ = state_tx.send(ConnectionState::Disconnected(id));
              break;
            },
          }
        },
        _ = task_context.wait_for_stop() => {
          break;
        },
      };
    }

    let _ = state_tx.send(ConnectionState::Closed(id));
  }

}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ConnectionId(usize);

impl ConnectionId {

  pub fn generate() -> Self {
    Self(random())
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
    ConnectionId(id)
  }

}