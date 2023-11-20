use std::collections::{hash_map::Entry, HashMap};
use std::error;
use std::fmt;
use std::sync::Arc;

use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::{Mutex, RwLock};

use video_rs::Error as MediaError;

use crate::media::sdp::{self, Sdp, SdpError};
use crate::media::MediaDescriptor;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::runtime::Runtime;
use crate::source::{
    self, Source, SourceDelegate, SourcePath, SourcePathRef, SourceState, SourceStateRx,
    SourceStateTx,
};

type SourceShared = Arc<Mutex<Source>>;
type SourceMap = Arc<RwLock<HashMap<SourcePath, SourceShared>>>;

type SourceDescriptionsCache = Arc<RwLock<HashMap<SourcePath, Sdp>>>;

pub struct SourceManager {
    sources: SourceMap,
    source_descriptions_cache: SourceDescriptionsCache,
    source_state_tx: SourceStateTx,
    worker: Task,
    runtime: Arc<Runtime>,
}

impl SourceManager {
    pub async fn start(runtime: Arc<Runtime>) -> Self {
        let sources = Arc::new(RwLock::new(HashMap::new()));
        let (source_state_tx, source_state_rx) = mpsc::unbounded_channel();

        let source_descriptions_cache = Arc::new(RwLock::new(HashMap::new()));

        tracing::trace!("starting source manager");
        let worker = runtime
            .task()
            .spawn({
                let sources = sources.clone();
                move |task_context| Self::run(sources.clone(), source_state_rx, task_context)
            })
            .await;
        tracing::trace!("started source manager");

        Self {
            sources,
            source_descriptions_cache,
            source_state_tx,
            worker,
            runtime,
        }
    }

    pub async fn stop(&mut self) {
        tracing::trace!("sending stop signal to source manager");
        self.worker.stop().await;
        tracing::trace!("stopped source manager");
        for (_, source) in self.sources.write().await.drain() {
            source.lock().await.stop().await;
        }
    }

    pub async fn register_and_start(
        &self,
        name: &str,
        path: SourcePath,
        descriptor: MediaDescriptor,
    ) -> Result<(), RegisterSourceError> {
        let path = source::normalize_path(path);
        let source = Source::start(
            name,
            path.clone(),
            descriptor,
            self.source_state_tx.clone(),
            self.runtime.as_ref(),
        )
        .await
        .map_err(RegisterSourceError::Media)?;

        if let Entry::Vacant(entry) = self.sources.write().await.entry(path.clone()) {
            let _ = entry.insert(Arc::new(Mutex::new(source)));
            tracing::trace!(name, %path, "registered and started source");
            tracing::trace!("requesting SDP for source to prime cache");
        } else {
            tracing::error!(name, %path, "source with given path already registered");
            return Err(RegisterSourceError::AlreadyRegistered);
        }

        self.describe(&path)
            .await
            .unwrap()
            .map_err(RegisterSourceError::Sdp)?;
        Ok(())
    }

    pub async fn describe(&self, path: &SourcePathRef) -> Option<Result<Sdp, SdpError>> {
        let cached_description = self
            .source_descriptions_cache
            .read()
            .await
            .get(path)
            .cloned();
        if let Some(description) = cached_description {
            tracing::trace!(%path, "pulled SDP from cache");
            Some(Ok(description))
        } else {
            let source = self.sources.read().await.get(path).cloned();
            if let Some(source) = source {
                let source_name = source.lock().await.name.clone();
                let source_descriptor = source.lock().await.descriptor.clone();
                let description = sdp::create(&source_name, &source_descriptor).await;
                if let Ok(description) = description.as_ref() {
                    self.source_descriptions_cache
                        .write()
                        .await
                        .insert(path.into(), description.clone());
                    tracing::trace!(%path, "cached SDP");
                }
                Some(description)
            } else {
                tracing::trace!(path, "tried to query SDP for source that does not exist");
                None
            }
        }
    }

    pub async fn subscribe(&self, path: &SourcePathRef) -> Option<SourceDelegate> {
        let source = self.sources.read().await.get(path).cloned();
        if let Some(source) = source {
            tracing::trace!(path, "creating source delegate for caller");
            Some(source.lock().await.delegate())
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
              // CANCEL SAFETY: `mpsc::UnboundedReceiver::recv` is cancel safe.
              state = source_state_rx.recv() => {
                match state {
                  Some(SourceState::Stopped(source_id)) => {
                    tracing::trace!(%source_id, "source manager: received stopped");
                    let _ = sources.write().await.remove(&source_id);
                  },
                  None => {
                    tracing::error!("source state channel broke unexpectedly");
                    break;
                  },
                }
              },
              // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
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
    Media(MediaError),
    Sdp(SdpError),
}

impl fmt::Display for RegisterSourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegisterSourceError::AlreadyRegistered => write!(f, "already registered"),
            RegisterSourceError::Media(err) => write!(f, "media error: {}", err),
            RegisterSourceError::Sdp(err) => write!(f, "sdp error: {}", err),
        }
    }
}

impl error::Error for RegisterSourceError {}
