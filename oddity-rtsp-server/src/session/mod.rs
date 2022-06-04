mod transport;

pub mod session_manager;
pub mod setup;

use std::fmt;

use tokio::select;
use tokio::sync::mpsc;

use rand::Rng;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::source::SourceDelegate;
use crate::session::setup::SessionSetup;

pub enum SessionState {
  Stopped(SessionId),
}

pub type SessionStateTx = mpsc::UnboundedSender<SessionState>;
pub type SessionStateRx = mpsc::UnboundedReceiver<SessionState>;

pub struct Session {
  worker: Task,
}

impl Session {

  pub async fn setup_and_start(
    id: SessionId,
    source_delegate: SourceDelegate,
    setup: SessionSetup,
    state_tx: SessionStateTx,
    runtime: &Runtime,
  ) -> Self {
    tracing::trace!(%id, "starting session");
    let worker = runtime
      .task()
      .spawn({
        let id = id.clone();
        |task_context| {
          Self::run(
            id,
            source_delegate,
            setup,
            state_tx,
            task_context,
          )
        }
      })
      .await;
    tracing::trace!(%id, "started session");

    Self {
      worker,
    }
  }

  pub async fn teardown(&mut self) {
    tracing::trace!("sending teardown signal to session");
    let _ = self.worker.stop().await;
    tracing::trace!("session torn down");
  }

  async fn run(
    id: SessionId,
    source_delegate: SourceDelegate,
    setup: SessionSetup,
    state_tx: SessionStateTx,
    mut task_context: TaskContext,
  ) {
    // TODO implement
    // TODO if the connection_sender_tx (inside setup) dies the it is
    // similar to transport being closed (underlying connection died)
    loop {
      select! {
        _ = task_context.wait_for_stop() => {
          tracing::trace!("tearing down session");
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