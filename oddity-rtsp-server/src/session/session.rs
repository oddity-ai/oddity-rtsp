use std::fmt;

use tokio::select;
use tokio::sync::mpsc;

use rand::Rng;

use oddity_rtsp_protocol::Transport;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};

pub enum SessionState {
  Stopped(SessionId),
}

pub type SessionStateTx = mpsc::UnboundedSender<SessionState>;
pub type SessionStateRx = mpsc::UnboundedReceiver<SessionState>;

pub enum SessionControlMessage {
  Teardown,
}

pub type SessionControlTx = mpsc::UnboundedSender<SessionControlMessage>;
pub type SessionControlRx = mpsc::UnboundedReceiver<SessionControlMessage>;

pub struct Session {
  control_tx: SessionControlTx,
  worker: Task,
}

impl Session {

  pub async fn start(
    id: SessionId,
    transport: Transport,
    state_tx: SessionStateTx,
    runtime: &Runtime,
  ) -> Self {
    let (control_tx, control_rx) = mpsc::unbounded_channel();

    let worker = runtime
      .task()
      .spawn(|task_context| {
        Self::run(
          id,
          control_rx,
          state_tx,
          task_context,
        )
      })
      .await;

    Self {
      control_tx,
      worker,
    }
  }

  pub async fn teardown(&mut self) {
    // First send a `Teardown` message to allow the session to be
    // tore down safely and resources be released.
    let _ = self.control_tx.send(SessionControlMessage::Teardown);
    // Use `Task` mechanism to send stop signal and stop join task.
    let _ = self.worker.stop().await;
  }

  pub fn control_tx(&self) -> SessionControlTx {
    self.control_tx.clone()
  }

  async fn run(
    id: SessionId,
    mut control_rx: SessionControlRx,
    state_tx: SessionStateTx,
    mut task_context: TaskContext,
  ) {
    // TODO if the connection_sender_tx (inside Transport) dies the it is
    // similar to transport being closed (underlying connection died)
    loop {
      select! {
        _ = control_rx.recv() => {
          break;
        },
        _ = task_context.wait_for_stop() => {
          break;
        },
      }
    }

    let _ = state_tx.send(SessionState::Stopped(id));
  }

}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
  const SESSION_ID_LEN: usize = 16;

  pub fn generate() -> SessionId {
    SessionId(
      rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(Self::SESSION_ID_LEN)
        .map(char::from)
        .collect()
    )
  }

}

impl fmt::Display for SessionId {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.0.fmt(f)
  }

}