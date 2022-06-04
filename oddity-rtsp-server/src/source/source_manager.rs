use std::fmt;
use std::error;
use std::sync::Arc;
use std::collections::{HashMap, hash_map::Entry};

use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::media::sdp::{self, Sdp, SdpError};
use crate::media::MediaDescriptor;
use crate::source::{
  Source,
  SourceDelegate,
  SourcePath,
  SourcePathRef,
  SourceState,
  SourceStateTx,
  SourceStateRx,
};

type SourceMap = Arc<Mutex<HashMap<SourcePath, Source>>>;

pub struct SourceManager {
  sources: SourceMap,
  source_state_tx: SourceStateTx,
  worker: Task,
  runtime: Arc<Runtime>,
}

impl SourceManager {

  pub async fn start(
    runtime: Arc<Runtime>,
  ) -> Self {
    let sources = Arc::new(Mutex::new(HashMap::new()));
    let (source_state_tx, source_state_rx) =
      mpsc::unbounded_channel();

    tracing::trace!("starting source manager");
    let worker = runtime
      .task()
      .spawn({
        let sources = sources.clone();
        move |task_context| {
          Self::run(
            sources.clone(),
            source_state_rx,
            task_context,
          )
        }
      })
      .await;
    tracing::trace!("started source manager");

    Self {
      sources,
      source_state_tx,
      worker,
      runtime,
    }
  }

  pub async fn stop(&mut self) {
    tracing::trace!("sending stop signal to source manager");
    self.worker.stop().await;
    tracing::trace!("stopped source manager");
    for (_, mut source) in self.sources.lock().await.drain() {
      source.stop().await;
    }
  }

  pub async fn register_and_start(
    &mut self,
    name: &str,
    path: SourcePath,
    descriptor: MediaDescriptor,
  ) -> Result<(), RegisterSourceError> {
    if let Entry::Vacant(entry) = self
        .sources
        .lock().await
        .entry(path.clone()) {
      let _ = entry.insert(
        Source::start(
          name.clone(),
          path.clone(),
          descriptor,
          self.source_state_tx.clone(),
          self.runtime.as_ref(),
        ).await
      );
      tracing::trace!(name, %path, "registered and started source");
      Ok(())
    } else {
      tracing::error!(name, %path, "source with given path already registered");
      Err(RegisterSourceError::AlreadyRegistered)
    }
  }

  pub async fn describe(
    &self,
    path: &SourcePathRef,
  ) -> Option<Result<Sdp, SdpError>> {
    if let Some(source) = self.sources.lock().await.get(path.into()) {
      Some(
        sdp::create(
          &source.name,
          &source.descriptor
        ).await
      )
    } else {
      tracing::trace!(path, "tried to query SDP for source that does not exist");
      None
    }
  }

  pub async fn subscribe(
    &self,
    path: &SourcePathRef,
  ) -> Option<SourceDelegate> {
    if let Some(source) = self.sources.lock().await.get_mut(path.into()) {
      tracing::trace!(path, "creating source delegate for caller");
      Some(source.delegate())
    } else {
      tracing::trace!(path, "tried to subscribe to source that does not exist");
      None
    }
  }

  async fn run(
    sources: SourceMap,
    mut source_state_rx: SourceStateRx,
    mut task_context: TaskContext,
  ) {
    loop {
      select! {
        state = source_state_rx.recv() => {
          match state {
            Some(SourceState::Stopped(source_id)) => {
              tracing::trace!(%source_id, "source manager: received stopped");
              let _ = sources.lock().await.remove(&source_id);
            },
            None => {
              tracing::error!("source state channel broke unexpectedly");
              break;
            },
          }
        },
        _ = task_context.wait_for_stop() => {
          tracing::trace!("stopping source manager");
          break;
        },
      }
    }
  }
  
}

#[derive(Debug)]
pub enum RegisterSourceError {
  AlreadyRegistered,
}

impl fmt::Display for RegisterSourceError {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RegisterSourceError::AlreadyRegistered => write!(f, "already registered"),
    }
  }

}

impl error::Error for RegisterSourceError {}