use std::fmt;
use std::error;
use std::sync::Arc;
use std::collections::{HashMap, hash_map::Entry};

use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use oddity_rtsp_protocol as rtsp;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::source::SourceDelegate;
use crate::session::setup::SessionSetup;
use crate::session::{
  Session,
  SessionId,
  SessionState,
  SessionStateTx,
  SessionStateRx,
  PlaySessionError,
};
use crate::media;

type SessionMap = Arc<Mutex<HashMap<SessionId, Session>>>;

pub struct SessionManager {
  sessions: SessionMap,
  session_state_tx: SessionStateTx,
  worker: Task,
  runtime: Arc<Runtime>,
}

impl SessionManager {

  pub async fn start(
    runtime: Arc<Runtime>,
  ) -> Self {
    let sessions = Arc::new(Mutex::new(HashMap::new()));
    let (session_state_tx, session_state_rx) =
      mpsc::unbounded_channel();

    tracing::trace!("starting session manager");
    let worker = runtime
      .task()
      .spawn({
        let sessions = sessions.clone();
        move |task_context| {
          Self::run(
            sessions.clone(),
            session_state_rx,
            task_context,
          )
        }
      })
      .await;
    tracing::trace!("started session manager");

    Self {
      sessions,
      session_state_tx,
      runtime,
      worker,
    }
  }

  pub async fn stop(&mut self) {
    tracing::trace!("sending stop signal to session manager");
    self.worker.stop().await;
    tracing::trace!("session manager stopped");
    for (_, mut session) in self.sessions.lock().await.drain() {
      session.teardown().await;
    }
  }

  pub async fn setup(
    &mut self,
    source_delegate: SourceDelegate,
    setup: SessionSetup,
  ) -> Result<SessionId, RegisterSessionError> {
    let session_id = SessionId::generate();
    if let Entry::Vacant(entry) = self
        .sessions
        .lock().await
        .entry(session_id.clone()) {
      let _ = entry.insert(
        Session::setup_and_start(
          session_id.clone(),
          source_delegate,
          setup,
          self.session_state_tx.clone(),
          self.runtime.as_ref(),
        ).await
      );
      tracing::trace!(%session_id, "registered new session");
      Ok(session_id)
    } else {
      tracing::error!(%session_id, "session with this ID already exists");
      Err(RegisterSessionError::AlreadyRegistered)
    }
  }

  pub async fn play(
    &mut self,
    id: &SessionId,
    range: Option<rtsp::Range>,
  ) -> Option<Result<media::StreamState, PlaySessionError>> {
    if let Some(session) = self.sessions.lock().await.get_mut(id) {
      tracing::trace!(session_id=%id, "start playing");
      Some(session.play(range).await)
    } else {
      tracing::trace!(
        session_id=%id,
        "caller tried to play session that does not exist",
      );
      None
    }
  }

  pub async fn teardown(
    &mut self,
    id: &SessionId,
  ) -> bool {
    if let Some(session) = self.sessions.lock().await.get_mut(id) {
      tracing::trace!(session_id=%id, "tearing down session");
      session.teardown().await;
      tracing::trace!(session_id=%id, "torn down session");
      true
    } else {
      tracing::trace!(
        session_id=%id,
        "caller tried to tear down session that does not exist",
      );
      false
    }
  }

  async fn run(
    sessions: SessionMap,
    mut session_state_rx: SessionStateRx,
    mut task_context: TaskContext,
  ) {
    loop {
      select! {
        // CANCEL SAFETY: `mpsc::UnboundedReceiver::recv` is cancel safe.
        state = session_state_rx.recv() => {
          match state {
            Some(SessionState::Stopped(session_id)) => {
              let _ = sessions.lock().await.remove(&session_id);
              tracing::trace!(%session_id, "session manager: received stopped");
            },
            None => {
              tracing::error!("session state channel broke unexpectedly");
              break;
            },
          }
        },
        // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
        _ = task_context.wait_for_stop() => {
          tracing::trace!("stopping session manager");
          break;
        },
      }
    }
  }
  
}

#[derive(Debug)]
pub enum RegisterSessionError {
  AlreadyRegistered,
}

impl fmt::Display for RegisterSessionError {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RegisterSessionError::AlreadyRegistered => write!(f, "already registered"),
    }
  }

}

impl error::Error for RegisterSessionError {}