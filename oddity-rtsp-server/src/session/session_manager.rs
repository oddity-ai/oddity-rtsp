use std::sync::Arc;
use std::collections::HashMap;

use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::session::setup::SessionSetup;
use crate::session::session::{
  Session,
  SessionId,
  SessionState,
  SessionStateTx,
  SessionStateRx,
};

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

    Self {
      sessions,
      session_state_tx,
      runtime,
      worker,
    }
  }

  pub async fn stop(&mut self) {
    self.worker.stop().await;
    // TODO move this into run???
    for (_, mut session) in self.sessions.lock().await.drain() {
      session.teardown().await;
    }
  }

  pub async fn setup_and_start(
    &mut self,
    setup: SessionSetup,
  ) {
    let session_id = SessionId::generate();
    let session = Session::setup_and_start(
        session_id,
        setup,
        self.session_state_tx.clone(),
        self.runtime.as_ref(),
      )
      .await;
  }

  pub async fn teardown(
    &mut self,
    id: &SessionId,
  ) {
    if let Some(session) = self.sessions.lock().await.get_mut(id) {
      session.teardown().await;
    } else {
      // TODO
    }
  }

  async fn run(
    sessions: SessionMap,
    mut session_state_rx: SessionStateRx,
    mut task_context: TaskContext,
  ) {
    loop {
      select! {
        state = session_state_rx.recv() => {
          match state {
            Some(SessionState::Stopped(session_id)) => {
              let _ = sessions.lock().await.remove(&session_id);
            },
            None => {
              // TODO
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