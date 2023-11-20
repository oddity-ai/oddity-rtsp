use std::collections::{hash_map::Entry, HashMap};
use std::error;
use std::fmt;
use std::sync::Arc;

use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, RwLock};

use oddity_rtsp_protocol as rtsp;

use crate::media;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::runtime::Runtime;
use crate::session::setup::SessionSetup;
use crate::session::{
    PlaySessionError, Session, SessionId, SessionState, SessionStateRx, SessionStateTx,
};
use crate::source::SourceDelegate;

type SessionShared = Arc<Mutex<Session>>;
type SessionMap = Arc<RwLock<HashMap<SessionId, SessionShared>>>;

pub struct SessionManager {
    sessions: SessionMap,
    session_state_tx: SessionStateTx,
    worker: Task,
    runtime: Arc<Runtime>,
}

impl SessionManager {
    pub async fn start(runtime: Arc<Runtime>) -> Self {
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let (session_state_tx, session_state_rx) = mpsc::unbounded_channel();

        tracing::trace!("starting session manager");
        let worker = runtime
            .task()
            .spawn({
                let sessions = sessions.clone();
                move |task_context| Self::run(sessions.clone(), session_state_rx, task_context)
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
        for (_, session) in self.sessions.write().await.drain() {
            session.lock().await.teardown().await;
        }
    }

    pub async fn setup(
        &self,
        source_delegate: SourceDelegate,
        setup: SessionSetup,
    ) -> Result<SessionId, RegisterSessionError> {
        let session_id = SessionId::generate();
        let session = Session::setup_and_start(
            session_id.clone(),
            source_delegate,
            setup,
            self.session_state_tx.clone(),
            self.runtime.as_ref(),
        )
        .await;

        if let Entry::Vacant(entry) = self.sessions.write().await.entry(session_id.clone()) {
            let _ = entry.insert(Arc::new(Mutex::new(session)));
            tracing::trace!(%session_id, "registered new session");
            Ok(session_id)
        } else {
            tracing::error!(%session_id, "session with this ID already exists");
            Err(RegisterSessionError::AlreadyRegistered)
        }
    }

    pub async fn play(
        &self,
        id: &SessionId,
        range: Option<rtsp::Range>,
    ) -> Option<Result<media::StreamState, PlaySessionError>> {
        let session = self.sessions.read().await.get(id).cloned();
        if let Some(session) = session {
            tracing::trace!(session_id=%id, "start playing");
            Some(session.lock().await.play(range).await)
        } else {
            tracing::trace!(
              session_id=%id,
              "caller tried to play session that does not exist",
            );
            None
        }
    }

    pub async fn teardown(&self, id: &SessionId) -> bool {
        let session = self.sessions.read().await.get(id).cloned();
        if let Some(session) = session {
            tracing::trace!(session_id=%id, "tearing down session");
            session.lock().await.teardown().await;
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
                    let _ = sessions.write().await.remove(&session_id);
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
